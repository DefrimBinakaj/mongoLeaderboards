// https://www.freecodecamp.org/news/mongodb-in-rust/
// https://www.mongodb.com/try/download/community
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused_mut, unused_variables, unused_imports, dead_code, unused_parens)]
use mongodb::{Client, bson::*, options::{ClientOptions, ResolverConfig, FindOptions}};
use std::env;
use std::error::Error;
use tokio;
use tokio_stream::StreamExt;

async fn create_database(client: &Client, db_name: &str) -> Result<(), Box<dyn Error>> {
   let _db = client.database(db_name);
   println!("Database {} created", db_name);
   Ok(())
}

async fn create_collection(client: &Client, db_name: &str, coll_name: &str) -> Result<(), Box<dyn Error>> {
   let db = client.database(db_name);
   let coll_names = db.list_collection_names(None).await?;

   if coll_names.contains(&coll_name.to_string()) {
       println!("Collection {} already exists in {}", coll_name, db_name);
   } else {
       let _coll = db.create_collection(coll_name, None).await?;
       println!("Collection {} created in {}", coll_name, db_name);
   }

   Ok(())
}

async fn insert_document(client: &Client, db_name: &str, coll_name: &str, doc_name: &str, doc_password: &str) -> Result<(), Box<dyn Error>> {
   let db = client.database(db_name);
   let coll = db.collection(coll_name);

   let doc = doc! { 
      "name": doc_name, 
      "password": doc_password,
      "c4gamesplayed": 0,
      "c4gameswon": 0,
      "tootgamesplayed": 0,
      "tootgameswon": 0,
   };

   coll.insert_one(doc, None).await.unwrap();

   println!("Document inserted with \nname:{}, \npassword:{} \n", doc_name, doc_password);
   println!("");
   Ok(())

}

async fn delete_document(client: &Client, db_name: &str, coll_name: &str, doc_name: &str) -> Result<(), Box<dyn Error>> {
   let db = client.database(db_name);
   let coll = db.collection::<Document>(coll_name);
   let filter = doc! { 
      "name": bson::Bson::String(doc_name.to_string()) 
   };

   // to delete only one entry with a specific name:
   // coll.delete_one(filter, None).await?;

   // to delete all entries with a specific name:
   coll.delete_many(filter, None).await?;
   
   println!("Document deleted!");
   Ok(())
}



async fn sign_in(client: &Client, db_name: &str, coll_name: &str, username: &str, password: &str) -> Result<(), Box<dyn Error>> {
   let db = client.database(db_name);
   let coll = db.collection::<Document>(coll_name);
   let filter = doc! {
       "name": bson::Bson::String(username.to_string()),
       "password": bson::Bson::String(password.to_string()),
   };
   let result = coll.find_one(filter, None).await?;

   if let Some(_) = result {
      println!("Welcome, {}! You are signed in.", username);
      let mut playgameInput = String::new();
      println!("Do you want to play a game? [y/n]");
      let mut playgameInputWrap = std::io::stdin().read_line(&mut playgameInput).unwrap();
      let mut playgameInputTrim = playgameInput.trim();
      if playgameInputTrim == "y" {
         play_game(&client, db_name, coll_name, username, password).await?;
      }
      else {
         println!("y no :(");
      }
   } else {
      println!("Invalid username or password. Please try again.");
   }

   Ok(())
}



// functions for manipulating document attributes (games played/won)
async fn play_game(client: &Client, db_name: &str, coll_name: &str, usernameInputTrim: &str, passwordInputTrim: &str) -> Result<(), Box<dyn Error>> {

   let mut playInput = String::new();
   println!("did you win? [cy / cn / ty / tn]");
   let mut playInputWrap = std::io::stdin().read_line(&mut playInput).unwrap();
   let mut playInputTrim = playInput.trim();

   if playInputTrim == "cy" {
      let db = client.database(db_name);
      let coll = db.collection::<Document>(coll_name);
      let filter = doc! {"name": bson::Bson::String(usernameInputTrim.to_string())};
      let update = doc! {
         "$inc": {
            "c4gamesplayed": 1,
            "c4gameswon": 1,
         },
      };
      let result = coll.update_many(filter, update, None).await?;
   }
   else if playInputTrim == "cn" {
      let db = client.database(db_name);
      let coll = db.collection::<Document>(coll_name);
      let filter = doc! {"name": bson::Bson::String(usernameInputTrim.to_string())};
      let update = doc! {
         "$inc": {
            "c4gamesplayed": 1,
         },
      };
      let result = coll.update_many(filter, update, None).await?;
   }
   else if playInputTrim == "ty" {
      let db = client.database(db_name);
      let coll = db.collection::<Document>(coll_name);
      let filter = doc! {"name": bson::Bson::String(usernameInputTrim.to_string())};
      let update = doc! {
         "$inc": {
            "tootgamesplayed": 1,
            "tootgameswon": 1,
         },
      };
      let result = coll.update_many(filter, update, None).await?;
   }
   else if playInputTrim == "tn" {
      let db = client.database(db_name);
      let coll = db.collection::<Document>(coll_name);
      let filter = doc! {"name": bson::Bson::String(usernameInputTrim.to_string())};
      let update = doc! {
         "$inc": {
            "tootgamesplayed": 1,
         },
      };
      let result = coll.update_many(filter, update, None).await?;
   }
   else {
      println!("invalid input");
   }

   Ok(())

}





// leaderboard for c4
// sorts by wins - then by winrate
// if winrate is NaN (due to 0 games played), then winrate is 0
async fn top_n_by_c4_games_played(client: &Client, db_name: &str, coll_name: &str, n: i64) -> Result<(), Box<dyn Error>> {
   let db = client.database(db_name);
   let coll = db.collection::<Document>(coll_name);

   let filter = doc! {};
   let find_options = FindOptions::builder()
       .sort(doc! { "c4gameswon": -1, "c4gamesplayed": 1 })
       .limit(n)
       .build();

   let mut cursor = coll.find(filter, find_options).await?;
   let mut top_n_documents = Vec::new();

   while let Some(result) = cursor.next().await {
       match result {
           Ok(document) => top_n_documents.push(document),
           Err(e) => return Err(Box::new(e)),
       }
   }

   println!("C4 champion players (top {}):", n);
   println!(" ");
   for (index, document) in top_n_documents.iter().enumerate() {
      if let (Some(name), Some(games_played), Some(games_won)) = (
         document.get("name").and_then(bson::Bson::as_str),
         document.get("c4gamesplayed").and_then(bson::Bson::as_i32),
         document.get("c4gameswon").and_then(bson::Bson::as_i32),
      ) 
      {
         let winrate = (games_won as f64 / games_played as f64) * 100.0;
         if winrate > 0.0 {
            println!("{}) {}:      WINS-{} PLAYED-{}    WINRATE-{:.0}%", index + 1, name, games_won, games_played, (games_won as f64 / games_played as f64) * 100.0 ); 
         }
         else {
            println!("{}) {}:      WINS-{} PLAYED-{}    WINRATE-{:.0}%", index + 1, name, games_won, games_played, 0.0 ); 
         }
      }
   }

   Ok(())
}




// leaderboard for toot
async fn top_n_by_toot_games_played(client: &Client, db_name: &str, coll_name: &str, n: i64) -> Result<(), Box<dyn Error>> {
   let db = client.database(db_name);
   let coll = db.collection::<Document>(coll_name);

   let filter = doc! {};
   let find_options = FindOptions::builder()
       .sort(doc! { "tootgameswon": -1, "tootgamesplayed": 1 })
       .limit(n)
       .build();

   let mut cursor = coll.find(filter, find_options).await?;
   let mut top_n_documents = Vec::new();

   while let Some(result) = cursor.next().await {
       match result {
           Ok(document) => top_n_documents.push(document),
           Err(e) => return Err(Box::new(e)),
       }
   }

   println!("TOOTOTTO champion players (top {}):", n);
   println!(" ");
   for (index, document) in top_n_documents.iter().enumerate() {
      if let (Some(name), Some(games_played), Some(games_won)) = (
         document.get("name").and_then(bson::Bson::as_str),
         document.get("tootgamesplayed").and_then(bson::Bson::as_i32),
         document.get("tootgameswon").and_then(bson::Bson::as_i32),
      )
      {
         let winrate = (games_won as f64 / games_played as f64) * 100.0;
         if winrate > 0.0 {
            println!("{}) {}:      WINS-{} PLAYED-{}    WINRATE-{:.0}%", index + 1, name, games_won, games_played, (games_won as f64 / games_played as f64) * 100.0 ); 
         }
         else {
            println!("{}) {}:      WINS-{} PLAYED-{}    WINRATE-{:.0}%", index + 1, name, games_won, games_played, 0.0 ); 
         }
      }
      // {
      //    println!("{}) {}:      WINS-{}   PLAYED-{}", index + 1, name, games_won, games_played ); 
      // }
   }

   Ok(())
}





#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

   let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
   let client = Client::with_options(client_options).unwrap();

   let db_name = "tempFromCompass";
   let coll_name = "players";

   // Print the databases in our MongoDB cluster:
   println!("Databases:");
   for name in client.list_database_names(None, None).await? {
      println!("- {}", name);
   }

   

   loop {

      println!("enter cmd: [done to exit]");
      println!("1 - insert doc");
      println!("2 - delete doc");
      println!("3 - sign in");
      println!("4 - leaderboards");

      let mut cmdNum = String::new();
      println!("choose command:");
      let mut cmdNumWrap = std::io::stdin().read_line(&mut cmdNum).unwrap();
      let mut cmdNumToString = cmdNum.trim();

      if cmdNumToString == "1" {
         println!("---");

         let mut usernameInput = String::new();
         println!("username:");
         let mut usernameInputWrap = std::io::stdin().read_line(&mut usernameInput).unwrap();
         let mut usernameInputTrim = usernameInput.trim();

         let mut passwordInput = String::new();
         println!("password:");
         let mut passwordInputWrap = std::io::stdin().read_line(&mut passwordInput).unwrap();
         let mut passwordInputTrim = passwordInput.trim();

         insert_document(&client, db_name, coll_name, usernameInputTrim, passwordInputTrim).await?;
         println!("---");
      }
      else if cmdNumToString == "2" {
         println!("---");

         let mut usernameInput = String::new();
         println!("username to delete:");
         let mut usernameInputWrap = std::io::stdin().read_line(&mut usernameInput).unwrap();
         let mut usernameInputTrim = usernameInput.trim();

         delete_document(&client, db_name, coll_name, usernameInputTrim).await?;
         println!("---");
      }
      else if cmdNumToString == "3" {
         println!("---");
         println!("what is your:");

         let mut usernameInput = String::new();
         println!(" - username:");
         let mut usernameInputWrap = std::io::stdin().read_line(&mut usernameInput).unwrap();
         let mut usernameInputTrim = usernameInput.trim();

         let mut passwordInput = String::new();
         println!(" - password:");
         let mut passwordInputWrap = std::io::stdin().read_line(&mut passwordInput).unwrap();
         let mut passwordInputTrim = passwordInput.trim();

         sign_in(&client, db_name, coll_name, usernameInputTrim, passwordInputTrim).await?;
         println!("---");
      }
      else if cmdNumToString == "4" {
         println!("---");
         top_n_by_c4_games_played(&client, db_name, coll_name, 4).await?;
         println!("---");
         top_n_by_toot_games_played(&client, db_name, coll_name, 3).await?;
         println!("---");
      }
      else if cmdNumToString == "done" {
         break;
      }
      else {
         println!("---");
         println!("invalid command");
         println!("---");
      }

   }





   // create_database(&client, "newDatabase").await?;
   // create_collection(&client, db_name, "alluserstemp").await?;
   
   

   Ok(())

}