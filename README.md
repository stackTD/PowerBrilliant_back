------------------------------------------------------------------------------------------
        Welcome to StackLeadership's Powerbrilliant's Backend repository.
------------------------------------------------------------------------------------------
PREREQUISITES:
- Rust [1.87 & above] using actix-web
- Postgres [17.5 & above]

    [Please refer to the PG_setup.txt & rust_setup.txt and  follow the instructions]

------------------------------------------------------------------------------------------
Instructions to start the server:
- Please install the prerequisites
- Please clone the repository 
- Please create the database and tables in Postgres using the queries given in folder "PG"
- **Set up environment variables** (see Environment Variables section below)
- Open the terminal and type the command: "cargo run"
- Once the server had started, please follow the API_documentation.docx

------------------------------------------------------------------------------------------
Environment Variables Setup:
------------------------------------------------------------------------------------------
This application uses environment variables for configuration. 

1. **Development Setup:**
   - The `.env` file contains development configuration
   - Update the values with your actual credentials:
     ```
     DATABASE_URL=postgres://postgres:your_password@localhost/powerbrilliant
     GOOGLE_CLIENT_ID=your_actual_google_client_id
     GOOGLE_CLIENT_SECRET=your_actual_google_client_secret
     ```

2. **Production Setup:**
   - Use the `.env.production` file as a template
   - Set `APP_ENV=production` to enable production mode
   - Configure all required environment variables for production

3. **Test Configuration:**
   - Run `cargo run --bin demo_config` to see current environment variables
   - This will show which variables are set and which are missing

4. **Required Variables:**
   - `DATABASE_URL` - PostgreSQL connection string
   - `GOOGLE_CLIENT_ID` - Google OAuth client ID  
   - `GOOGLE_CLIENT_SECRET` - Google OAuth client secret
   - `OAUTH_REDIRECT_URL` - OAuth callback URL
   - Other optional variables (see ENVIRONMENT_VARIABLES.md for full list)

For detailed documentation, see **ENVIRONMENT_VARIABLES.md**