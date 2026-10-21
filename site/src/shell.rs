use stucco::layout::{Container, SkipLink};
use stucco::theme::Preset;
use stucco::{Attrs, Bundle, Meta, Page, Render, el};

use crate::DOCUMENTS;

pub(crate) struct Site {
    pub base: String,
    pub bundle: Bundle,
}

impl Site {
    pub fn new(base: String) -> Self {
        let bundle = Bundle::new(Preset::Sand).prefix(&format!("{base}_stucco"));
        Self { base, bundle }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    pub fn page<'a>(
        &self,
        title: &str,
        description: &str,
        current: &str,
        content: impl Render + 'a,
    ) -> String {
        let title = if current == "home" {
            "Symposiast — guidance for the release you use".to_owned()
        } else {
            format!("{title} · Symposiast")
        };
        let head = (
            el::link()
                .attr("rel", "icon")
                .attr("type", "image/svg+xml")
                .href(self.url("assets/favicon.svg")),
            el::script()
                .attr("type", "module")
                .attr("src", self.url("assets/site.js")),
        );
        Page::new(&self.bundle, &title)
            .meta(Meta::description(description).og_title(&title))
            .head(head)
            .body_attrs(Attrs::default().class("site-body"))
            .body((
                SkipLink::new(),
                self.header(current),
                el::main()
                    .id("main")
                    .attr("tabindex", "-1")
                    .child(Container::new().class("site-container").child(content)),
                self.footer(),
            ))
            .render()
    }

    fn header(&self, current: &str) -> impl Render + 'static {
        let mut nav = el::nav().class("site-nav").aria("label", "Main navigation");
        for (slug, title, _) in DOCUMENTS {
            let mut link = el::a().href(self.url(&format!("{slug}.html"))).text(title);
            if current == slug {
                link = link.aria("current", "page");
            }
            nav = nav.child(link);
        }
        nav = nav.child(
            el::a()
                .href("https://mcaveniathor.github.io/symposiast-registry/")
                .text("Registry"),
        );
        el::header().class("site-header").child(
            Container::new()
                .class("site-container header-inner")
                .child(
                    el::a()
                        .class("wordmark")
                        .href(self.url(""))
                        .aria("label", "Symposiast home")
                        .child(
                            el::span()
                                .class("brand-mark")
                                .aria("hidden", "true")
                                .text("s."),
                        )
                        .child(el::span().text("symposiast")),
                )
                .child(nav)
                .child(
                    el::a()
                        .class("source-link")
                        .href("https://github.com/mcaveniathor/symposiast")
                        .text("GitHub"),
                ),
        )
    }

    fn footer(&self) -> impl Render + 'static {
        el::footer().class("site-footer").child(
            Container::new()
                .class("site-container footer-inner")
                .child(
                    el::div()
                        .child(el::p().class("footer-name").text("symposiast"))
                        .child(el::p().text("Crate knowledge, with a version attached.")),
                )
                .child(
                    el::nav()
                        .aria("label", "Related projects")
                        .child(
                            el::a()
                                .href(self.url("maintainers.html#contribute-to-the-project"))
                                .text("Contribute"),
                        )
                        .child(
                            el::a()
                                .href("https://mcaveniathor.github.io/symposiast-registry/")
                                .text("Registry"),
                        )
                        .child(
                            el::a()
                                .href("https://github.com/mcaveniathor/symposiast")
                                .text("Source"),
                        )
                        .child(
                            el::a()
                                .href("https://github.com/symposium-dev/symposium")
                                .text("Symposium"),
                        )
                        .child(
                            el::a()
                                .href("https://github.com/mcaveniathor/stucco")
                                .text("Built with Stucco"),
                        ),
                ),
        )
    }
}
