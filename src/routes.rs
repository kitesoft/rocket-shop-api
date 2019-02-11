use rocket_contrib::databases::postgres;
use rocket_contrib::json::{Json, JsonValue};

use super::model;

#[database("postgres_db")]
struct MyDatabase(postgres::Connection);

#[derive(Serialize, Deserialize)]
struct CreateUser {
    email: String,
    first_name: String,
    last_name: String,
    access_token: String,
    user_id: i64,
}

#[derive(Serialize, Deserialize)]
struct CreateCategory {
    name: String,
    display_name: String,
    parent_id: i64,
}

#[derive(Serialize, Deserialize)]
struct Category {
    name: String,
    display_name: String,
    children: Vec<Category>,
}

///
/// curl -X POST -H "Content-Type:application/json" -d '{"email":"batman@cave.com","first_name":"Bruce","last_name":"Wayne","access_token":"pass","user_id","1"}' http://localhost:8000/user
///
#[post("/user", format = "json", data = "<data>")]
fn create_user(data: Json<CreateUser>, conn: MyDatabase) -> JsonValue {
    let user = model::User {
        id: None,
        email: data.email.clone(),
        first_name: data.first_name.clone(),
        last_name: data.last_name.clone(),
        access_token: data.access_token.clone(),
        user_id: data.user_id,
    };

    let user = model::User::create(user, &conn);
    json!({ "status": "ok", "user_id": user.id.unwrap()})
}

///
/// curl -X POST -H "Content-Type:application/json" -d '{"name":"cars","display_name":"Batmobile"}' http://localhost:8000/categories
///
#[post("/categories", format = "json", data = "<data>")]
fn create_category(data: Json<CreateCategory>, conn: MyDatabase) -> JsonValue {
    let new_category = model::Category {
        id: None,
        name: data.name.clone(),
        display_name: data.display_name.clone(),
        parent_id: Some(data.parent_id.clone()),
    };

    let new_category = model::Category::create(new_category, &conn);

    json!({ "status": "ok", "id" : new_category.id.unwrap()})
}

#[get("/categories")]
fn list_category(conn: MyDatabase) -> JsonValue {
    let categories = list_category_by_parent(None, &conn);
    json!({"status": "ok","categories":categories})
}

fn list_category_by_parent(parent_id: Option<i64>, conn: &postgres::Connection) -> Vec<Category> {
    model::Category::list_by_parent(parent_id, conn)
        .iter()
        .map(|cat| Category {
            name: cat.name.clone(),
            display_name: cat.display_name.clone(),
            children: list_category_by_parent(cat.id, conn),
        })
        .collect()
}

pub fn launch() {
    rocket::ignite()
        .attach(MyDatabase::fairing())
        .mount("/api", routes![create_user, create_category, list_category])
        .launch();
}
