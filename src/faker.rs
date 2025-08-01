use fake::faker::{
    internet::en::{SafeEmail, Username, Password},
    name::en::{FirstName, LastName},
    phone_number::en::CellNumber,
    company::en::CompanyName,
    lorem::en::{Sentence, Paragraph},
};
use fake::faker::job::en::Position;
use fake::faker::boolean::en::Boolean;
use fake::{Fake, Faker};
use rand::seq::SliceRandom;
use uuid::Uuid;
use sqlx::PgPool;
use fake::faker::address::en::CityName;
use chrono::Utc;
use serde_json::json;
 
pub async fn seed_fake_users(pool: &PgPool, count: usize) {
    for _ in 0..count {
        let id = Uuid::new_v4();
        let first_name: String = FirstName().fake();
        let last_name: String = LastName().fake();
        let username: String = Username().fake();
        let password: String = Password(10..16).fake();
        let email: String = SafeEmail().fake();
        let mobile: String = CellNumber().fake();
        let organisation: String = CompanyName().fake();
        let bio = json!({
            "about": Sentence(8..15).fake::<String>()
        });
        let bio_str = bio.to_string();
        let profile_pic = format!("https://api.dicebear.com/6.x/identicon/svg?seed={}", username);
        let provider = "local";
        let provider_user_id = username.clone();
        let access_token = None::<String>;
        let refresh_token = None::<String>;
        let oauth_email = None::<String>;
        let created_at = Utc::now().naive_utc();
        let updated_at = created_at;
        let is_active: Option<bool> = Some(Boolean(50).fake::<bool>());
 
        let result = sqlx::query!(
            r#"
            INSERT INTO users (
                id, first_name, last_name, username, password,
                mobile, email, organisation, profile_pic,
                created_at, updated_at, is_active, bio,
                provider, provider_user_id, access_token,
                refresh_token, oauth_email
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12, $13,
                $14, $15, $16,
                $17, $18
            )
            "#,
            id,
            first_name,
            last_name,
            username,
            password,
            Some(mobile),
            email,
            Some(organisation),
            Some(profile_pic),
            created_at,
            updated_at,
            is_active,
          Some(&bio),// <-- fix here
            Some(provider),
            Some(provider_user_id),
            access_token,
            refresh_token,
            oauth_email,
        )
        .execute(pool)
        .await;
 
        match result {
            Ok(_) => println!("✅ Created user: {username}"),
            Err(e) => println!("⚠️ Error inserting user {}: {:?}", username, e),
        }
    }
}
 
 
pub async fn seed_fake_communities(pool: &PgPool, count: usize) {
    // Fetch all user IDs
    let user_ids: Vec<Uuid> = match sqlx::query_scalar!("SELECT id FROM users")
        .fetch_all(pool)
        .await
    {
        Ok(ids) => ids,
        Err(e) => {
            println!("⚠️ Error fetching user ids: {:?}", e);
            return;
        }
    };
 
    if user_ids.is_empty() {
        println!("⚠️ No users found. Cannot create communities without users.");
        return;
    }
 
    for _ in 0..count {
        let id = Uuid::new_v4();
        let name: String = CompanyName().fake();
        let description = json!({
            "about": Sentence(8..15).fake::<String>()
        });
        let creator_id = user_ids[rand::random::<usize>() % user_ids.len()];
        let banner_url = Some(format!("https://picsum.photos/seed/{}/600/200", id));
        let is_public = Boolean(80).fake::<bool>(); // 80% chance true
        let is_active = Boolean(90).fake::<bool>(); // 90% chance true
        let created_at = Utc::now().naive_utc();
 
        let result = sqlx::query!(
            r#"
            INSERT INTO communities (
                id, name, description, creator_id, banner_url,
                is_public, is_active, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            id,
            name,
            description,
            creator_id,
            banner_url,
            is_public,
            is_active,
            created_at,
        )
        .execute(pool)
        .await;
 
        match result {
            Ok(_) => println!("✅ Created community: {name}"),
            Err(e) => println!("⚠️ Error inserting community {}: {:?}", name, e),
        }
    }
}
 
 
pub async fn seed_fake_posts(pool: &PgPool, count: usize) {
    // Fetch all user IDs
    let user_ids: Vec<Uuid> = match sqlx::query_scalar!("SELECT id FROM users")
        .fetch_all(pool)
        .await
    {
        Ok(ids) => ids,
        Err(e) => {
            println!("⚠️ Error fetching user ids: {:?}", e);
            return;
        }
    };
 
    // Fetch all community IDs
    let community_ids: Vec<Uuid> = match sqlx::query_scalar!("SELECT id FROM communities")
        .fetch_all(pool)
        .await
    {
        Ok(ids) => ids,
        Err(e) => {
            println!("⚠️ Error fetching community ids: {:?}", e);
            return;
        }
    };
 
    if user_ids.is_empty() || community_ids.is_empty() {
        println!("⚠️ No users or communities found. Cannot create posts without them.");
        return;
    }
 
    for _ in 0..count {
        let id = Uuid::new_v4();
        let author_id = user_ids[rand::random::<usize>() % user_ids.len()];
        let community_id = community_ids[rand::random::<usize>() % community_ids.len()];
        let title = json!(Sentence(3..8).fake::<String>());
        let content = json!(Paragraph(2..5).fake::<String>());
        let post_type = "text".to_string();
        let created_at = Utc::now().naive_utc();
        let is_active = Boolean(90).fake::<bool>();
 
        let result = sqlx::query!(
            r#"
            INSERT INTO posts (
                id, community_id, author_id, title, content, post_type, created_at, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            id,
            community_id,
            author_id,
            title,
            content,
            post_type,
            created_at,
            is_active,
        )
        .execute(pool)
        .await;
 
        match result {
            Ok(_) => println!("✅ Created post in community {community_id} by user {author_id}"),
            Err(e) => println!("⚠️ Error inserting post: {:?}", e),
        }
    }
}
 
 
pub async fn seed_fake_jobs(pool: &PgPool, count: usize) {
    // Fetch all user IDs
    let user_ids: Vec<Uuid> = match sqlx::query_scalar!("SELECT id FROM users")
        .fetch_all(pool)
        .await
    {
        Ok(ids) => ids,
        Err(e) => {
            println!("⚠️ Error fetching user ids: {:?}", e);
            return;
        }
    };
 
    if user_ids.is_empty() {
        println!("⚠️ No users found. Cannot create jobs without users.");
        return;
    }
 
    let job_types = vec!["remote", "hybrid", "onsite"];
    let skills_pool = vec![
        "Rust", "SQL", "React", "AWS", "Docker", "Kubernetes", "Python", "TypeScript", "Node.js"
    ];
 
    // Use the provided valid business account UUID for posted_by
    let job_posted_by = Uuid::parse_str("bff17406-d7d9-4821-8314-a2ad5eb28253").unwrap();

    for _ in 0..count {
        let id = Uuid::new_v4();
        let organisation: String = CompanyName().fake();
        let title = json!(Position().fake::<String>());
        let description = json!(Paragraph(2..5).fake::<String>());
        let location: String = CityName().fake();
        let job_type = Some(job_types.choose(&mut rand::thread_rng()).unwrap().to_string());
        let notice_period = Some(format!("{} weeks", (1..4).fake::<u8>()));
        let min_experience = Some((0..10).fake::<i32>());
        let salary_range = Some(format!("{}-{} USD", (40..80).fake::<u32>(), (81..150).fake::<u32>()));
        let skills: Vec<String> = skills_pool
            .choose_multiple(&mut rand::thread_rng(), 3)
            .cloned()
            .map(String::from)
            .collect();
        let created_at = Utc::now().naive_utc();
        let updated_at = created_at;
        let is_active = Some(true);
       let job_posted_by = Uuid::parse_str("bff17406-d7d9-4821-8314-a2ad5eb28253").unwrap();

        let result = sqlx::query!(
            r#"
            INSERT INTO job_listings (
                id, organisation, title, description, location, job_type, skills, notice_period,
                min_experience, salary_range, created_at, updated_at, is_active, posted_by
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14
            )
            "#,
            id,
            organisation,
            title,
            description,
            location,
            job_type,
            &skills,
            notice_period,
            min_experience,
            salary_range,
            created_at,
            updated_at,
            is_active,
            job_posted_by, // <-- always use the valid business account UUID
        )
        .execute(pool)
        .await;
 
        match result {
            Ok(_) => println!("✅ Created job: {organisation} - {title}"),
            Err(e) => println!("⚠️ Error inserting job: {:?}", e),
        }
    }
}