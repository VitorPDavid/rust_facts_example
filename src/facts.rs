use std::{collections::HashMap, fmt::Display};

#[derive(Debug)]
#[derive(Clone)]
pub struct Fact {
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
    pub fn new(entity: String, attribute: String, value: FactValue, operation: bool) -> Fact {
        Fact {
            attribute: attribute,
            entity: entity,
            value: value,
            operation: operation
        }
    }

    pub fn to_file(&self) -> String {
        return format!("{}---{}---{}---True", self.entity, self.attribute, self.value);
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct SchemaFact {
    entity: String,
    attribute: String,
    value: String,
}

impl SchemaFact {
    pub fn new(entity: String, attribute: String, value: String) -> SchemaFact {
        SchemaFact {
            entity,
            attribute,
            value
        }
    }
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub enum FactValue {
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

pub fn write_facts_to_file(facts: &Vec<Fact>, file_path: &str) {
    let mut content: String = String::from("");
    
    for fact in facts {
        content = format!("{}{}\n", content, fact.to_file());
    }

    std::fs::write(file_path, content).expect("Error reading {file_path}");
}

pub fn get_schema_from_file(file_path: &str) -> Vec<SchemaFact> {
    let facts_schema_text = std::fs::read_to_string(file_path).expect("Error reading {file_path}");

    let mut schema_fact: Vec<SchemaFact> = vec![];

    for line in facts_schema_text.lines() {
        let line_split: Vec<&str> = line.split("---").collect();
        schema_fact.push(SchemaFact::new(line_split[0].to_string(), line_split[1].to_string(), line_split[2].to_string()))
    }

    schema_fact
}

pub fn get_facts_from_file(file_path: &str) -> Vec<Fact> {
    let facts_text = std::fs::read_to_string(file_path).expect("Error reading {file_path}");

    let mut facts: Vec<Fact> = vec![];

    for line in facts_text.lines() {
        let line_split: Vec<&str> = line.split("---").collect();

        let value: FactValue = match line_split[2].parse::<i64>() {
            Ok(value) => FactValue::IntValue(value),
            Err(_) => FactValue::StringValue(line_split[2].to_string()),
        };

        let operation: bool = match line_split[3] {
            "False" => false,
            "false" => false,
            "True" => true,
            "true" => true,
            _ => false,
        };

        facts.push(Fact::new(line_split[0].to_string(), line_split[1].to_string(), value, operation))
    }

    facts
}

pub fn get_active_facts(facts: &Vec<Fact>, schema: &Vec<SchemaFact>) -> Vec<Fact> {
    let mut attributes_one: HashMap<&str, HashMap<&str, &FactValue>> = HashMap::new();
    let mut attributes_many: HashMap<&str, HashMap<&str, Vec<&FactValue>>> = HashMap::new();

    for fact_index in 0..facts.len() {
        let fact = &facts[fact_index];
        let is_many = attribute_is_many(&fact.attribute, &schema);

        match is_many {
            false => handle_one_attribute(fact, &mut attributes_one),
            true => handle_many_attribute(fact, &mut attributes_many),
        }
    }

    let mut active_facts= Vec::new();
    active_facts = get_attributes_one(&mut attributes_one, active_facts);
    active_facts = get_attributes_many(&mut attributes_many, active_facts);

    active_facts
}

fn attribute_is_many(attribute: &str, schema: &Vec<SchemaFact>) -> bool {
    for fact_schema in schema.iter() {
        if fact_schema.entity == attribute && fact_schema.attribute == "cardinality" {
            return fact_schema.value == "many";
        }
    }

    false
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
