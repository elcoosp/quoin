use gpui::*;
use quoin_macros::{component, quoin_render};

#[derive(Clone)]
pub struct Entry {
    pub key: String,
    pub value: String,
    pub hits: u32,
}

component! {
    pub SortableTableTest {
        state {
            sort_col: String = String::new(),
            sort_dir: String = "asc".to_string(),
            entries: Vec<Entry> = vec![
                Entry { key: "name".to_string(), value: "Alice".to_string(), hits: 42 },
                Entry { key: "age".to_string(), value: "30".to_string(), hits: 17 },
                Entry { key: "city".to_string(), value: "NYC".to_string(), hits: 99 },
            ],
        }

        fn do_sort(col: &str, dir: &str) {
            sort_col.set(col.to_string());
            sort_dir.set(dir.to_string());
        }

        render {
            let entries = entries.get();
            quoin_render! {
                data_table(
                    rows: entries,
                    striped: true,
                    on_sort: move |col, dir| do_sort(col, dir),
                ) {
                    column(key: "key",   label: "Key",   sortable: true, width: 120.0,
                           render: |row: &Entry| row.key.clone())
                    column(key: "value", label: "Value", sortable: true, width: 200.0,
                           render: |row: &Entry| row.value.clone())
                    column(key: "hits",  label: "Hits",  sortable: true, width: 80.0,
                           render: |row: &Entry| row.hits.to_string())
                }
            }
        }
    }
}

fn main() {}
