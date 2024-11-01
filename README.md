# Knifer
A self-hosted demo analysis tool

## Usage
### Environment Variables
- `DATABASE_URL`
- `STEAM_API_KEY`
- `BASE_URL`

If using the 's3' storage backend
- `S3_ACCESS_KEY`
- `S3_SECRET_KEY`
- `S3_REGION`
- `S3_ENDPOINT`
- `S3_BUCKET`

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
