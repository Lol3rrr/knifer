# Knifer
A self-hosted demo analysis tool

## Usage
### Environment Variables
- `DATABASE_URL`
- `STEAM_API_KEY`

### Needed external Software
- `postgresql`


## Development
### Frontend
1. Navigate to the frontend folder
2. Run `trunk watch`

### Backend
1. Navigate to the root folder
2. Run `cargo run --bin backend`

### DB Stuff
We use [diesel]() as the ORM and using the cli for all the migrations
