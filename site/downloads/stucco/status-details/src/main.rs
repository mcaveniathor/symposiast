use stucco::Delivery;
use stucco::prelude::*;

fn main() {
    let bundle = Bundle::new(Preset::Slate);
    let content = Stack::new()
        .child(
            PageHeader::new("Order 1042").breadcrumbs(
                Breadcrumbs::new()
                    .link("Orders", "/orders")
                    .current("Order 1042"),
            ),
        )
        .child(Cluster::of((
            Avatar::new("Ada Lovelace").decorative(),
            Text::new("Ada Lovelace"),
        )))
        .child(
            DescriptionList::new()
                .item("Status", Badge::new("Paid").tone(Tone::Success))
                .item("Total", "$128.00"),
        )
        .child(
            Stat::new("Orders", "128")
                .change("12%", Trend::Up)
                .change_tone(Tone::Success)
                .context("vs last week"),
        )
        .child(
            Steps::new("Checkout")
                .step("Cart", "/checkout/cart")
                .step("Delivery", "/checkout/delivery")
                .step_disabled("Payment")
                .current(1),
        )
        .child(Progress::new("Import", 42.0, 100.0).show_value())
        .child(
            Meter::new("Storage", 7.2, 0.0, 10.0)
                .low(3.0)
                .high(8.0)
                .optimum(1.0)
                .value_text("7.2 of 10 GB"),
        )
        .child(
            Callout::new(Tone::Info)
                .title("Preparing for dispatch")
                .child(Text::new("Your order will ship within two working days.")),
        )
        .child(
            Disclosure::new("Shipping details")
                .heading(2)
                .child(Text::new("Tracked delivery is included.")),
        )
        .child(
            Accordion::new()
                .heading(2)
                .item_open("Shipping", Text::new("Arrives by Friday."))
                .item("Returns", Text::new("Return within 30 days."))
                .exclusive(),
        );
    let html = Page::new(&bundle, "Order 1042")
        .delivery(Delivery::Inline)
        .main(content)
        .render();
    for marker in [
        "<progress",
        "<meter",
        "<details",
        "<summary",
        "Paid",
        "Order 1042",
        "$128.00",
    ] {
        assert!(html.contains(marker), "missing {marker}");
    }
    println!("{html}");
}
