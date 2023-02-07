use std::{collections::HashMap, fmt::Display};

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
enum FactValue {
    StringValue(String),
    IntValue(i64),
}

impl Display for FactValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FactValue::StringValue(x) => write!(f, "{}", x),
            FactValue::IntValue(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
struct Fact {
    attribute: String,
    entity: String,
    value: FactValue,
    operation: bool,
}

impl PartialEq for Fact {
    fn eq(&self, other: &Fact) -> bool {
        if self.entity == other.entity && self.attribute == other.attribute {
            return match &self.value {
                FactValue::StringValue(_) => match &other.value {
                    FactValue::StringValue(_) => self.value == other.value,
                    _ => return false,
                },
                FactValue::IntValue(_) => match &other.value {
                    FactValue::IntValue(_) => self.value == other.value,
                    _ => return false,
                },
            }
        }
        false
    }
}

impl Display for Fact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.entity, self.attribute, self.value)
    }
}

impl Fact {
    fn new(entity: String, attribute: String, value: FactValue, operation: bool) -> Fact {
        Fact {
            attribute: attribute,
            entity: entity,
            value: value,
            operation: operation
        }
    }
}


#[derive(Debug)]
#[derive(Clone)]
struct SchemaFact {
    entity: String,
    attribute: String,
    value: String,
}

impl SchemaFact {
    fn new(entity: String, attribute: String, value: String) -> SchemaFact {
        SchemaFact {
            entity,
            attribute,
            value
        }
    }
}

fn main() {
    let facts: Vec<Fact> = vec![
        Fact::new("entity/1".to_string(), "label".to_string(), FactValue::StringValue("primeira".to_string()), true),
        Fact::new("entity/1".to_string(), "depth".to_string(), FactValue::IntValue(10), true),
        Fact::new("entity/1".to_string(), "depth".to_string(), FactValue::IntValue(25), true),
        Fact::new("entity/2".to_string(), "label".to_string(), FactValue::StringValue("segunda".to_string()), true),
        Fact::new("entity/2".to_string(), "test".to_string(), FactValue::StringValue("abc".to_string()), true),
        Fact::new("entity/2".to_string(), "test".to_string(), FactValue::StringValue("ab".to_string()), true),
        Fact::new("entity/2".to_string(), "test".to_string(), FactValue::StringValue("ab".to_string()), false),
        Fact::new("entity/1".to_string(), "test".to_string(), FactValue::StringValue("abacate".to_string()), true),
        Fact::new("entity/1".to_string(), "test".to_string(), FactValue::StringValue("abacate".to_string()), false),
        Fact::new("entity/2".to_string(), "depth".to_string(), FactValue::IntValue(25), true),
        Fact::new("entity/2".to_string(), "depth".to_string(), FactValue::IntValue(15), true),
        Fact::new("entity/2".to_string(), "depth".to_string(), FactValue::IntValue(15), false),
    ];

    let schema_fact = vec![
        SchemaFact::new("label".to_string(), "cardinality".to_string(), "one".to_string()),
        SchemaFact::new("depth".to_string(), "cardinality".to_string(), "one".to_string()),
        SchemaFact::new("test".to_string(), "cardinality".to_string(), "many".to_string()),
    ];

    let active_facts = valid_facts(facts, schema_fact);

    for fact in active_facts {
        println!("{}\n", fact);
    }
}

fn valid_facts(facts: Vec<Fact>, schema: Vec<SchemaFact>) -> Vec<Fact> {
    let mut attributes_one: HashMap<&str, HashMap<&str, &FactValue>> = HashMap::new();
    let mut attributes_many: HashMap<&str, HashMap<&str, Vec<&FactValue>>> = HashMap::new();

    for fact_index in 0..facts.len() {
        let fact = &facts[fact_index];
        let cardinality_option = get_attribute_cardinality(&fact.attribute, &schema);

        if let Some(cardinality) = cardinality_option {
            match cardinality {
                "one" => handle_one_attribute(fact, &mut attributes_one),
                "many" => handle_many_attribute(fact, &mut attributes_many),
                _ => (),
            }
        }
    }

    let mut active_facts= Vec::new();
    active_facts = get_attributes_one(&mut attributes_one, active_facts);
    active_facts = get_attributes_many(&mut attributes_many, active_facts);

    active_facts
}

fn get_attribute_cardinality<'a>(attribute: &str, schema: &'a Vec<SchemaFact>) -> Option<&'a str> {
    for fact_schema in schema.iter() {
        if fact_schema.entity == attribute && fact_schema.attribute == "cardinality" {
            return Some(&fact_schema.value);
        }
    }

    None
}

fn handle_one_attribute<'a>(fact: &'a Fact, attributes_one: &mut HashMap<&'a str, HashMap<&'a str, &'a FactValue>>) {
    match fact.operation {
        true => {
            match attributes_one.get_mut(&fact.entity[..]) {
                Some(attributes) => {
                    attributes.insert(&fact.attribute, &fact.value);
                },
                None => {
                    let mut attribute_map = HashMap::new();
                    attribute_map.insert(&fact.attribute[..], &fact.value);

                    attributes_one.insert(&fact.entity[..], attribute_map);
                }
            }
        },
        false => {
            if let Some(attributes) = attributes_one.get_mut(&fact.entity[..]) {
                if let Some(&actual_value) = attributes.get(&fact.attribute[..]) {
                    if *actual_value == fact.value {
                        attributes.remove(&fact.attribute[..]);
                    }
                }
            }
        },
    }
}

fn handle_many_attribute<'a>(fact: &'a Fact, attributes_many: &mut HashMap<&'a str, HashMap<&'a str, Vec<&'a FactValue>>>) {
    match fact.operation {
        true => {
            match attributes_many.get_mut(&fact.entity[..]) {
                Some(attributes) => {
                    match attributes.get_mut(&fact.attribute[..]) {
                        Some(actual_values) => {
                            actual_values.push(&fact.value);
                        },
                        None => {
                            let mut values = Vec::new();
                            values.push(&fact.value);

                            attributes.insert(&fact.attribute[..], values);
                        },
                    }
                },
                None => {
                    let mut attribute_map = HashMap::new();
                    let mut values = Vec::new();

                    values.push(&fact.value);
                    attribute_map.insert(&fact.attribute[..], values);
                    attributes_many.insert(&fact.entity[..], attribute_map);
                }
            }
        },
        false => {
            if let Some(attributes) = attributes_many.get_mut(&fact.entity[..]) {
                if let Some(actual_values) = attributes.get_mut(&fact.attribute[..]) {
                    actual_values.retain(|x| **x != fact.value);
                }
            }
        },
    }
}

fn get_attributes_one(attributes_one: &mut HashMap<&str, HashMap<&str, &FactValue>>, mut active_facts: Vec<Fact>) -> Vec<Fact> {
    for (&entity, attribute_map) in &*attributes_one {
        for (&attribute, &value) in &*attribute_map {
            active_facts.push(Fact::new(entity.to_string(), attribute.to_string(), value.to_owned(), true));
        }
    }

    active_facts
}

fn get_attributes_many(attributes_many: &mut HashMap<&str, HashMap<&str, Vec<& FactValue>>>, mut active_facts: Vec<Fact>) -> Vec<Fact> {
    for (&entity, attribute_map) in attributes_many.iter_mut() {
        for (&attribute, values) in &*attribute_map {
            for &value in values {
                active_facts.push(Fact::new(entity.to_string(), attribute.to_string(), value.to_owned(), true));
            }
        }
    }

    active_facts
}
