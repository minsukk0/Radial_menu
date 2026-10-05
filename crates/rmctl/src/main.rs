use clap::{Parser, Subcommand};
use radial_core::{
    deserialize_item_for_kind, generate_schema, generate_unique_id, get_default_config_path,
    read_config, update_config, validate_config, Category, CategoryKind, Item, StoreError,
    ValidationError,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(
    name = "rmctl",
    about = "Radial Menu CLI configuration management tool",
    version
)]
struct Cli {
    #[arg(long, global = true, help = "Pretty-print JSON output")]
    pretty: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Output the active menu.json configuration path
    Path,

    /// Output the JSON Schema for menu.json
    Schema,

    /// Validate menu configuration
    Validate {
        #[arg(long, help = "Path to menu.json file to validate (defaults to active config)")]
        file: Option<PathBuf>,
    },

    /// List categories and items
    List {
        #[arg(long, help = "Filter by category ID")]
        category: Option<String>,
    },

    /// Manage categories
    Category {
        #[command(subcommand)]
        command: CategoryCommands,
    },

    /// Manage items
    Item {
        #[command(subcommand)]
        command: ItemCommands,
    },
}

#[derive(Subcommand, Debug)]
enum CategoryCommands {
    /// Add a new category
    Add {
        #[arg(long, help = "Category kind: app | terminal | folder | system")]
        kind: String,

        #[arg(long, help = "Category label")]
        label: String,

        #[arg(long, help = "Optional custom category ID")]
        id: Option<String>,

        #[arg(long, help = "Position index to insert at")]
        at: Option<usize>,
    },

    /// Remove a category
    Remove {
        #[arg(help = "Category ID to remove")]
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum ItemCommands {
    /// Add a new item
    Add {
        #[arg(long, help = "Category ID or kind (app | terminal | folder | system)")]
        category: String,

        #[arg(long, help = "Item JSON string")]
        json: Option<String>,

        #[arg(long, help = "Path to JSON file containing item")]
        file: Option<PathBuf>,

        #[arg(long, help = "Position index to insert at")]
        at: Option<usize>,

        #[arg(long, help = "Create category if kind was specified and does not exist")]
        create_category: bool,
    },

    /// Update an existing item
    Update {
        #[arg(help = "Item ID to update")]
        id: String,

        #[arg(long, help = "JSON string containing fields to update")]
        json: Option<String>,

        #[arg(long, help = "Path to JSON file containing fields to update")]
        file: Option<PathBuf>,
    },

    /// Remove an item
    Remove {
        #[arg(help = "Item ID to remove")]
        id: String,
    },

    /// Move an item to a new position or category
    Move {
        #[arg(help = "Item ID to move")]
        id: String,

        #[arg(long, help = "Target position index")]
        to: usize,

        #[arg(long, help = "Target category ID (optional)")]
        category: Option<String>,
    },
}

fn print_success(val: Value, pretty: bool) {
    if pretty {
        println!("{}", serde_json::to_string_pretty(&val).unwrap());
    } else {
        println!("{}", serde_json::to_string(&val).unwrap());
    }
    std::process::exit(0);
}

fn print_validation_error(errors: Vec<ValidationError>, pretty: bool) -> ! {
    let err_json = json!({
        "ok": false,
        "errors": errors
    });
    if pretty {
        println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
    } else {
        println!("{}", serde_json::to_string(&err_json).unwrap());
    }
    std::process::exit(1);
}

fn print_usage_error(path: &str, message: &str, pretty: bool) -> ! {
    let err_json = json!({
        "ok": false,
        "errors": [
            {
                "path": path,
                "message": message
            }
        ]
    });
    if pretty {
        println!("{}", serde_json::to_string_pretty(&err_json).unwrap());
    } else {
        println!("{}", serde_json::to_string(&err_json).unwrap());
    }
    std::process::exit(2);
}

fn handle_store_error(err: StoreError, pretty: bool) -> ! {
    match err {
        StoreError::Validation(errors) => print_validation_error(errors, pretty),
        other => print_usage_error("", &other.to_string(), pretty),
    }
}

fn main() {
    let cli = Cli::parse();
    let pretty = cli.pretty;

    match cli.command {
        Commands::Path => {
            let path = get_default_config_path();
            print_success(
                json!({
                    "ok": true,
                    "path": path.to_string_lossy()
                }),
                pretty,
            );
        }
        Commands::Schema => {
            let schema = generate_schema();
            print_success(
                json!({
                    "ok": true,
                    "schema": schema
                }),
                pretty,
            );
        }
        Commands::Validate { file } => {
            let path = file.unwrap_or_else(get_default_config_path);
            let (config, _) = match read_config(&path) {
                Ok(res) => res,
                Err(e) => handle_store_error(e, pretty),
            };

            match validate_config(&config) {
                Ok(()) => {
                    print_success(
                        json!({
                            "ok": true,
                            "message": "설정이 유효합니다."
                        }),
                        pretty,
                    );
                }
                Err(errors) => {
                    print_validation_error(errors, pretty);
                }
            }
        }
        Commands::List { category } => {
            let path = get_default_config_path();
            let (config, _) = match read_config(&path) {
                Ok(res) => res,
                Err(e) => handle_store_error(e, pretty),
            };

            if let Some(cat_id) = category {
                let found = config.categories.iter().find(|c| c.id == cat_id);
                match found {
                    Some(cat) => {
                        print_success(
                            json!({
                                "ok": true,
                                "category": cat,
                                "items": cat.items
                            }),
                            pretty,
                        );
                    }
                    None => {
                        print_usage_error("", &format!("분류를 찾을 수 없습니다: '{cat_id}'"), pretty);
                    }
                }
            } else {
                print_success(
                    json!({
                        "ok": true,
                        "categories": config.categories
                    }),
                    pretty,
                );
            }
        }
        Commands::Category { command } => match command {
            CategoryCommands::Add {
                kind,
                label,
                id,
                at,
            } => {
                let parsed_kind = match CategoryKind::from_str(&kind) {
                    Ok(k) => k,
                    Err(msg) => print_usage_error("/kind", &msg, pretty),
                };

                let mut created_id = String::new();
                let mut created_cat: Option<Category> = None;

                let res = update_config(None, |config| {
                    let mut existing_ids: HashSet<String> = config
                        .categories
                        .iter()
                        .map(|c| c.id.clone())
                        .collect();

                    let final_id = match id.filter(|s| !s.trim().is_empty()) {
                        Some(custom) => custom,
                        None => generate_unique_id(&label, &existing_ids),
                    };

                    created_id = final_id.clone();
                    existing_ids.insert(final_id.clone());

                    let cat = Category::new(final_id, parsed_kind, label);
                    created_cat = Some(cat.clone());

                    let pos = at.unwrap_or(config.categories.len());
                    if pos < config.categories.len() {
                        config.categories.insert(pos, cat);
                    } else {
                        config.categories.push(cat);
                    }

                    Ok(())
                });

                match res {
                    Ok(_) => {
                        print_success(
                            json!({
                                "ok": true,
                                "id": created_id,
                                "category": created_cat
                            }),
                            pretty,
                        );
                    }
                    Err(e) => handle_store_error(e, pretty),
                }
            }
            CategoryCommands::Remove { id } => {
                let mut removed = false;
                let res = update_config(None, |config| {
                    let prev_len = config.categories.len();
                    config.categories.retain(|c| c.id != id);
                    if config.categories.len() < prev_len {
                        removed = true;
                        Ok(())
                    } else {
                        Err(StoreError::Custom(format!("삭제할 분류를 찾을 수 없습니다: '{id}'")))
                    }
                });

                match res {
                    Ok(_) if removed => {
                        print_success(
                            json!({
                                "ok": true,
                                "removedId": id
                            }),
                            pretty,
                        );
                    }
                    Ok(_) => {
                        print_usage_error("", &format!("삭제할 분류를 찾을 수 없습니다: '{id}'"), pretty);
                    }
                    Err(e) => handle_store_error(e, pretty),
                }
            }
        },
        Commands::Item { command } => match command {
            ItemCommands::Add {
                category,
                json,
                file,
                at,
                create_category,
            } => {
                let json_str = match (json, file) {
                    (Some(s), None) => s,
                    (None, Some(f)) => match std::fs::read_to_string(&f) {
                        Ok(s) => s,
                        Err(e) => print_usage_error("/file", &format!("파일 읽기 실패: {e}"), pretty),
                    },
                    (Some(_), Some(_)) => {
                        print_usage_error("", "--json과 --file을 동시에 지정할 수 없습니다.", pretty)
                    }
                    (None, None) => {
                        print_usage_error("", "--json 또는 --file 중 하나를 지정해야 합니다.", pretty)
                    }
                };

                let json_clean = json_str.trim_start_matches('\u{feff}');
                let item_val: Value = match serde_json::from_str(json_clean) {
                    Ok(v) => v,
                    Err(e) => print_usage_error("/json", &format!("유효한 JSON이 아닙니다: {e}"), pretty),
                };

                let mut added_id = String::new();
                let mut target_cat_id = String::new();
                let mut added_item: Option<Item> = None;
                let mut needs_approval = false;

                let res = update_config(None, |config| {
                    // Find target category
                    let mut found_cat_idx = config.categories.iter().position(|c| c.id == category);

                    if found_cat_idx.is_none() {
                        if let Ok(kind) = CategoryKind::from_str(&category) {
                            found_cat_idx = config.categories.iter().position(|c| c.kind == kind);
                            if found_cat_idx.is_none() {
                                if create_category {
                                    let mut existing: HashSet<String> = config
                                        .categories
                                        .iter()
                                        .map(|c| c.id.clone())
                                        .collect();
                                    let new_id = generate_unique_id(kind.as_str(), &existing);
                                    existing.insert(new_id.clone());
                                    let new_cat = Category::new(new_id, kind, kind.default_label());
                                    config.categories.push(new_cat);
                                    found_cat_idx = Some(config.categories.len() - 1);
                                } else {
                                    return Err(StoreError::Custom(format!(
                                        "'{category}' 종류의 분류가 없습니다. 새로 생성하려면 --create-category를 지정하세요."
                                    )));
                                }
                            }
                        }
                    }

                    let cat_idx = found_cat_idx.ok_or_else(|| {
                        StoreError::Custom(format!("분류를 찾을 수 없습니다: '{category}'"))
                    })?;

                    let target_kind = config.categories[cat_idx].kind;
                    target_cat_id = config.categories[cat_idx].id.clone();

                    let mut item = deserialize_item_for_kind(target_kind, &item_val)
                        .map_err(|e| StoreError::Custom(format!("항목 역직렬화 실패: {e}")))?;

                    if item.id().trim().is_empty() {
                        let existing_ids: HashSet<String> = config
                            .categories
                            .iter()
                            .flat_map(|c| c.items.iter().map(|i| i.id().to_string()))
                            .collect();
                        let new_id = generate_unique_id(item.label(), &existing_ids);
                        item.set_id(new_id);
                    }

                    added_id = item.id().to_string();
                    needs_approval = item.run_as_admin();
                    added_item = Some(item.clone());

                    let cat = &mut config.categories[cat_idx];
                    let insert_pos = at.unwrap_or(cat.items.len());
                    if insert_pos < cat.items.len() {
                        cat.items.insert(insert_pos, item);
                    } else {
                        cat.items.push(item);
                    }

                    Ok(())
                });

                match res {
                    Ok(_) => {
                        print_success(
                            json!({
                                "ok": true,
                                "id": added_id,
                                "categoryId": target_cat_id,
                                "needsApproval": needs_approval,
                                "item": added_item
                            }),
                            pretty,
                        );
                    }
                    Err(e) => handle_store_error(e, pretty),
                }
            }
            ItemCommands::Update { id, json, file } => {
                let json_str = match (json, file) {
                    (Some(s), None) => s,
                    (None, Some(f)) => match std::fs::read_to_string(&f) {
                        Ok(s) => s,
                        Err(e) => print_usage_error("/file", &format!("파일 읽기 실패: {e}"), pretty),
                    },
                    (Some(_), Some(_)) => {
                        print_usage_error("", "--json과 --file을 동시에 지정할 수 없습니다.", pretty)
                    }
                    (None, None) => {
                        print_usage_error("", "--json 또는 --file 중 하나를 지정해야 합니다.", pretty)
                    }
                };

                let json_clean = json_str.trim_start_matches('\u{feff}');
                let patch_val: Value = match serde_json::from_str(json_clean) {
                    Ok(v) => v,
                    Err(e) => print_usage_error("/json", &format!("유효한 JSON이 아닙니다: {e}"), pretty),
                };

                let mut updated_item: Option<Item> = None;
                let mut needs_approval = false;

                let res = update_config(None, |config| {
                    for cat in &mut config.categories {
                        for item in &mut cat.items {
                            if item.id() == id {
                                item.update_from_json(&patch_val)
                                    .map_err(StoreError::Custom)?;
                                needs_approval = item.run_as_admin();
                                updated_item = Some(item.clone());
                                return Ok(());
                            }
                        }
                    }
                    Err(StoreError::Custom(format!("수정할 항목을 찾을 수 없습니다: '{id}'")))
                });

                match res {
                    Ok(_) => {
                        print_success(
                            json!({
                                "ok": true,
                                "id": id,
                                "needsApproval": needs_approval,
                                "item": updated_item
                            }),
                            pretty,
                        );
                    }
                    Err(e) => handle_store_error(e, pretty),
                }
            }
            ItemCommands::Remove { id } => {
                let mut found = false;

                let res = update_config(None, |config| {
                    for cat in &mut config.categories {
                        let prev_len = cat.items.len();
                        cat.items.retain(|i| i.id() != id);
                        if cat.items.len() < prev_len {
                            found = true;
                            return Ok(());
                        }
                    }
                    Err(StoreError::Custom(format!("삭제할 항목을 찾을 수 없습니다: '{id}'")))
                });

                match res {
                    Ok(_) if found => {
                        print_success(
                            json!({
                                "ok": true,
                                "removedId": id
                            }),
                            pretty,
                        );
                    }
                    Ok(_) => {
                        print_usage_error("", &format!("삭제할 항목을 찾을 수 없습니다: '{id}'"), pretty);
                    }
                    Err(e) => handle_store_error(e, pretty),
                }
            }
            ItemCommands::Move { id, to, category } => {
                let mut target_cat_id = String::new();
                let mut final_idx = 0;

                let res = update_config(None, |config| {
                    let mut found_item: Option<(usize, usize, Item)> = None;

                    'outer: for (cat_idx, cat) in config.categories.iter().enumerate() {
                        for (item_idx, item) in cat.items.iter().enumerate() {
                            if item.id() == id {
                                found_item = Some((cat_idx, item_idx, item.clone()));
                                break 'outer;
                            }
                        }
                    }

                    let (src_cat_idx, src_item_idx, item) = found_item.ok_or_else(|| {
                        StoreError::Custom(format!("이동할 항목을 찾을 수 없습니다: '{id}'"))
                    })?;

                    let dest_cat_idx = if let Some(dest_id) = &category {
                        let idx = config.categories.iter().position(|c| c.id == *dest_id).ok_or_else(|| {
                            StoreError::Custom(format!("대상 분류를 찾을 수 없습니다: '{dest_id}'"))
                        })?;
                        if config.categories[idx].kind != item.kind() {
                            return Err(StoreError::Custom(format!(
                                "대상 분류 '{}'의 종류({:?})가 항목의 종류({:?})와 일치하지 않습니다.",
                                dest_id, config.categories[idx].kind, item.kind()
                            )));
                        }
                        idx
                    } else {
                        src_cat_idx
                    };

                    target_cat_id = config.categories[dest_cat_idx].id.clone();

                    if src_cat_idx == dest_cat_idx {
                        let items = &mut config.categories[src_cat_idx].items;
                        let item = items.remove(src_item_idx);
                        let insert_idx = to.min(items.len());
                        items.insert(insert_idx, item);
                        final_idx = insert_idx;
                    } else {
                        config.categories[src_cat_idx].items.remove(src_item_idx);
                        let dest_items = &mut config.categories[dest_cat_idx].items;
                        let insert_idx = to.min(dest_items.len());
                        dest_items.insert(insert_idx, item);
                        final_idx = insert_idx;
                    }

                    Ok(())
                });

                match res {
                    Ok(_) => {
                        print_success(
                            json!({
                                "ok": true,
                                "id": id,
                                "newIndex": final_idx,
                                "categoryId": target_cat_id
                            }),
                            pretty,
                        );
                    }
                    Err(e) => handle_store_error(e, pretty),
                }
            }
        },
    }
}
