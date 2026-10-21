use stucco::prelude::*;
use stucco::{CollectionPage, to_html};

#[derive(Columns)]
struct Order {
    #[col(label = "ID", sortable)]
    id: u64,
    #[col(sortable, searchable)]
    customer: String,
    #[col(key = "total", label = "Total", value = dollars, display = money)]
    total_cents: u64,
}

fn dollars(order: &Order) -> f64 {
    order.total_cents as f64 / 100.0
}

fn money(order: &Order) -> String {
    format!(
        "${}.{:02}",
        order.total_cents / 100,
        order.total_cents % 100
    )
}

fn main() {
    let page = CollectionPage {
        rows: vec![Order {
            id: 7,
            customer: "<Ada>".into(),
            total_cents: 4800,
        }],
        next: None,
        prev: None,
        total: None,
    };
    let table = DataTable::from_page(&page, "Orders").columns(Order::columns());
    let html = to_html(&table);
    assert!(html.contains("&lt;Ada&gt;"));
    assert!(html.contains("$48.00"));
    println!("{html}");
}
