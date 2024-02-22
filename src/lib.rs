use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RStr, RString, RVec},
};
use mexprp::{ParseError, Term};
use quick_search_lib::{ColoredChar, PluginId, SearchLib, SearchLib_Ref, SearchResult, Searchable, Searchable_TO};

static NAME: &str = "Math";

#[export_root_module]
pub fn get_library() -> SearchLib_Ref {
    SearchLib { get_searchable }.leak_into_prefix()
}

#[sabi_extern_fn]
fn get_searchable(id: PluginId) -> Searchable_TO<'static, RBox<()>> {
    let this = Math::new(id);
    Searchable_TO::from_value(this, TD_Opaque)
}

#[derive(Debug, Clone)]
struct Math {
    id: PluginId,
}

impl Searchable for Math {
    fn search(&self, query: RString) -> RVec<SearchResult> {
        let mut res = vec![];

        if let Ok::<Term<f64>, ParseError>(term) = mexprp::Term::parse(query.as_str()) {
            // if it parses, then we can evaluate it
            match term.eval() {
                Ok(mexprp::Answer::Multiple(xs)) => {
                    for x in xs {
                        res.push(result(term.clone(), x));
                    }
                }
                Ok(mexprp::Answer::Single(x)) => {
                    res.push(result(term.clone(), x));
                }
                Err(e) => {
                    log::error!("Error evaluating expression: {:?}", e);
                }
            }
        }

        res.sort_by(|a, b| a.title().cmp(b.title()));
        res.dedup_by(|a, b| a.title() == b.title());

        res.into()
    }
    fn name(&self) -> RStr<'static> {
        NAME.into()
    }
    fn colored_name(&self) -> RVec<quick_search_lib::ColoredChar> {
        // can be dynamic although it's iffy how it might be used
        ColoredChar::from_string(NAME, 0x16BE2FFF)
    }
    fn execute(&self, result: &SearchResult) {
        let s = result.extra_info();
        if let Ok::<clipboard::ClipboardContext, Box<dyn std::error::Error>>(mut clipboard) = clipboard::ClipboardProvider::new() {
            if let Ok(()) = clipboard::ClipboardProvider::set_contents(&mut clipboard, s.to_owned()) {
                log::info!("copied to clipboard: {}", s);
            } else {
                log::error!("failed to copy to clipboard: {}", s);
            }
        } else {
            log::error!("failed to copy to clipboard: {}", s);
        }
    }
    fn plugin_id(&self) -> PluginId {
        self.id.clone()
    }
}

fn result(term: Term<f64>, res: f64) -> SearchResult {
    let resstr = format!("{} = {}", term, res);
    SearchResult::new(&resstr).set_extra_info(&format!("{}", res))
}

impl Math {
    fn new(id: PluginId) -> Self {
        Self { id }
    }
}
