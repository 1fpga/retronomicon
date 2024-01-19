# Contributing

> [!IMPORTANT]
> These instructions expect you to have a working development environment, ideally Linux or MacOS.
> If you are using Windows, you will need to change the commands accordingly.

This guide explains how to setup a development environment for Retronomicon.

## Prerequisites
To work on Retronomicon, you will need the following tools:
- [Git](https://git-scm.com/downloads)
- [Docker](https://docs.docker.com/get-docker/)

> [!WARNING]
> This guide also uses `jq` to parse JSON files.
> It is not necessary to work on Retronomicon, but it is used in this guide.

To work on the backend, you will need the following tools:
- [Rust](https://www.rust-lang.org/tools/install)

To work on the frontend, you will need the following tools:
- [Node.js](https://nodejs.org/en/download/)
- [Yarn](https://classic.yarnpkg.com/en/docs/install) or [npm](https://www.npmjs.com/get-npm). 
  The commands below use `npm`, but you can replace them with `yarn` if you prefer.

## Setup
To setup your development environment, you will need to clone the repository and build the Docker images.

### Clone the repository
To clone the repository, run the following command:
```bash
git clone https://github.com/golem-fpga/retronomicon.git
```

### Build the dependencies
The backend requires a database and an instance of MinIO to run.
You can run them locally, but the easiest way is to use Docker as well.

If you're using Docker for dependencies, re-starting the containers will lose all data, so it's recommended to create a volume for their data.
In this example I create a directory in my home directory, but you can use any directory you want.
```bash
mkdir -p ~/temp/retronomicon-data/{postgresql,minio}
```

To start the dependencies, run the following commands:
```bash
docker run --name pgsql-dev -d \
           -e POSTGRES_USER=local_user \
           -e POSTGRES_PASSWORD=mysecretpassword \
           -e POSTGRES_DB=local_retronomicon \
           -e PGDATA=/var/lib/postgresql/data/pgdata \
           -p 127.0.0.1:5432:5432 \
           -v $HOME/temp/retronomicon-data/postgresql:/var/lib/postgresql/data \
           postgres

docker run --name minio-dev -d \
           -p 127.0.0.1:9000:9000 \
           -p 127.0.0.1:9001:9001 \
           --user $(id -u):$(id -g) \
           -e "MINIO_ROOT_USER=root" \
           -e "MINIO_ROOT_PASSWORD=changeme123" \
           -v $HOME/temp/retronomicon-data/minio:/data \
           quay.io/minio/minio server /data --console-address ":9001"
```

### Configuring the MinIO instance
The MinIO instance needs to be configured with a bucket and a policy.
Keys need to be generated as well.

Open a browser and go to http://localhost:9001.
Login with the credentials you set in the `docker run` command above (in this guide, the username is `root` and its password is `changeme123`.

Create 3 buckets:
- `retronomicon-cores`.
  This will contain globally accessible files for downloading cores.
- `retronomicon-games`.
  This will contain globally accessible files for games (e.g. box arts).
- `retronomicon-users`.
  This will contain user-specific files (e.g. save states and screenshots).

For local development, since the docker instance isn't accessible from outside your computer if using the docker commands above, you can set the bucket policies to `Public`.
Click on each bucket to change their access policy.

Next, you need to create API access keys.
Go to the `Access Keys` tab and click on `Create access key` button in the top right, then click on `Create`.

Make sure to copy the access key and secret key, as you will need them later.
You can also download the keys as a JSON file, and run the following commands to extract the keys:
```bash
export AWS_ACCESS_KEY_ID=$(cat Credentials.json | jq -r '.accessKey')
export AWS_SECRET_ACCESS_KEY=$(cat Credentials.json | jq -r '.secretKey')
```

Don't worry too much about the keys and/or policies, as the MinIO instance is only accessible from your computer.

### Build the Backend
You can use `cargo` if you're developing for the backend.

If not, I would suggest using Docker to build and run the backend.
To build the Docker images, run the following command:
```bash
docker build -t retronomicon-backend
```

This will create a Docker image named `retronomicon-backend` that contains the backend.
Run it with the following command (assuming you created the database and MinIO containers using the commands above):

> [!IMPORTANT]
> The access keys for AWS must be changed to the right values generated above in the "Configuring the MinIO instance" section.

> [!IMPORTANT]
> If you don't configure an OAUTH client, you won't be able to login to the backend (yet).
> Creating users is not implemented yet.
> I suggest using GitHub as an OAUTH provider, as it's the easiest to setup.
> For more information, see [Configuring OAUTH on GitHub](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authenticating-to-the-rest-api-with-an-oauth-app).

```bash
export AWS_ACCESS_KEY_ID="THE ACCESS KEY GENERATED ABOVE"
export AWS_SECRET_ACCESS_KEY="THE SECRET KEY GENERATED ABOVE"

export ROCKET_OAUTH_GITHUB_CLIENT_ID="YOUR GITHUB CLIENT ID"
export ROCKET_OAUTH_GITHUB_CLIENT_SECRET="YOUR GITHUB CLIENT SECRET"

# This will add additional users to the root team.
export ROCKET_DEBUG_ROOT_ADDITIONAL_EMAIL="ANY EMAIL YOU WANT AS ROOT USER"

# Get the IP address of the containers above.
# Change this to your IP address if you're not using Docker.
export DATABASE_URL="postgres://local_user:mysecretpassword@$(docker inspect pgsql-dev | jq -r '.[0].NetworkSettings.IPAddress'):5432/local_retronomicon"
export AWS_REGION="$(docker inspect minio-dev | jq -r '.[0].NetworkSettings.IPAddress'):9000"

docker run -it --rm \
    -e ROCKET_SECRET_KEY=(openssl rand -base64 64 | tr -d '\n') \
    -e DATABASE_URL=$DATABASE_URL \
    -e AWS_REGION=$AWS_REGION \
    -e AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID \
    -e AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY \
    -e ROCKET_OAUTH_GITHUB_CLIENT_ID=$ROCKET_OAUTH_GITHUB_CLIENT_ID \
    -e ROCKET_OAUTH_GITHUB_CLIENT_SECRET=$ROCKET_OAUTH_GITHUB_CLIENT_SECRET \
    -e ROCKET_DEBUG_ROOT_ADDITIONAL_EMAIL=$ROCKET_DEBUG_ROOT_ADDITIONAL_EMAIL \
    -e RUST_BACKTRACE=1 \
    -e ROCKET_BASE_URL=http://localhost:3000/ \
    -p 127.0.0.1:8000:8000 \
    retronomicon-backend
```

> [!NOTE]
> The `ROCKET_SECRET_KEY` environment variable is used to encrypt the session cookies.
> It is generated randomly in the command above, but you can set it to any value you want.
> If you keep that value random, you will need to set it again if you restart the container, and cookies are going to be invalidated on restart.

## Developing for the Backend

### Build the Frontend
If you are running the backend with Docker, it will automatically build and include the frontend.

But if you are running the backend with `cargo`, you will need to build the frontend separately at least once.

To build the frontend, run the following command:
```bash
npm install
npm run -w frontend build
```

The output will be in the `frontend/build` directory.
Make sure to have a STATIC_ROOT environment variable pointing to that directory when running the backend.

### Environment variables
The backend uses the following environment variables:
- `ROCKET_SECRET_KEY`:
  The secret key used to encrypt session cookies.
- `DATABASE_URL`: 
  The URL of the PostgreSQL database.
- `AWS_REGION`: 
  The URL of the MinIO instance.
- `AWS_ACCESS_KEY_ID`:
  The access key ID for the MinIO instance.
- `AWS_SECRET_ACCESS_KEY`:
  The secret access key for the MinIO instance.
- `STATIC_ROOT`: 
  The directory containing the frontend static files to serve.
- `ROCKET_OAUTH_{PROVIDER}_CLIENT_ID`: 
  The OAuth client ID for the given provider (in ALL CAPS). 
  Supported providers are `github`, `google` and `patreon`.
- `ROCKET_OAUTH_{PROVIDER}_CLIENT_SECRET`: 
  The OAuth client secret for the given provider (in ALL CAPS). 
  Supported providers are `github`, `google` and `patreon`.
- 

## Developing for the Frontend
If you're not working on the frontend, you can skip this step.

Make sure to have the proper API URL .
```bash
export REACT_APP_API_URL=http://localhost:8000
```

To start the frontend, run the following command:
```bash
npm start -w frontend
```
