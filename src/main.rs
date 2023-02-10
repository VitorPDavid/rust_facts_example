pub mod facts;

use crate::facts::*;

fn main() {
    let schema_fact = get_schema_from_file("./files/schema.txt");
    let facts = get_facts_from_file("./files/facts.txt");

    let active_facts = get_active_facts(&facts, &schema_fact);

    write_facts_to_file(&active_facts, "./files/output_rust.txt");
}
