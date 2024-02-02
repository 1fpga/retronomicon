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
export ROCKET_S3__ACCESS_KEY=$(cat credentials.json | jq -r '.accessKey')
export ROCKET_S3__SECRET_KEY=$(cat credentials.json | jq -r '.secretKey')
```

Don't worry too much about the keys and/or policies, as the MinIO instance is only accessible from your computer.

### Build the Backend
You can use `cargo` if you're developing for the backend.

If not, I would suggest using Docker to build and run the backend.
To build the Docker images, run the following command:
```bash
docker build -t retronomicon-backend .
```

This will create a Docker image named `retronomicon-backend` that contains the backend.
Run it with the following command (assuming you created the database and MinIO containers using the commands above):

> [!IMPORTANT]
> The access keys for AWS must be changed to the right values generated above in the "Configuring the MinIO instance" section.

> [!NOTE]
> Environment variables prefixed with `ROCKET_` will supersede configuration values in `Rocket.toml`.
> Their name is namespaced by `__` (double underscore) instead of `.` (dot) to avoid issues with environment variables on some platforms.
> For example, the `ROCKET_S3__ACCESS_KEY` environment variable will override the `s3.access_key` value in `Rocket.toml`.

```bash
export ROCKET_S3__ACCESS_KEY="THE ACCESS KEY GENERATED ABOVE"
export ROCKET_S3__SECRET_KEY="THE SECRET KEY GENERATED ABOVE"
# This key needs to be the same between restarts of the server.
# When running locally this is set to a static value in `Rocket.debug.toml`,
# But that file is not used in docker.
export ROCKET_SECRET_KEY="dH+kbvuRgr6z/OQaycGZEjMFKRFnhBlJJha9CYnWCNNpnsGHSGcOb+HZsmwLGoOf84Xz5d1EGMT/1EnVJxoDFw=="

# This will add additional users to the root team.
# This has to be an array of strings. Any wildcard (*, ?) will be interpreted.
export ROCKET_DEBUG__ADDITIONAL_ROOT_TEAM='[]'

# Get the IP address of the containers above.
# Change this to your IP address if you're not using Docker.
export ROCKET_DATABASES__RETRONOMICON_DB__URL="postgres://local_user:mysecretpassword@$(docker inspect pgsql-dev | jq -r '.[0].NetworkSettings.IPAddress'):5432/local_retronomicon"
export ROCKET_S3__REGION="$(docker inspect minio-dev | jq -r '.[0].NetworkSettings.IPAddress'):9000"

# If you just want to run Retronomicon locally without your own frontend, you
# can use `http://localhost:8000` instead.
export ROCKET_BASE_URL="http://localhost:3000/"

docker run -it --rm \
    -e ROCKET_SECRET_KEY \
    -e ROCKET_S3__ACCESS_KEY \
    -e ROCKET_S3__SECRET_KEY \
    -e ROCKET_S3__REGION \
    -e ROCKET_DEBUG__ADDITIONAL_ROOT_TEAM \
    -e ROCKET_DATABASES__RETRONOMICON_DB__URL \
    -e ROCKET_BASE_URL \
    -p 127.0.0.1:8000:8000 \
    retronomicon-backend
```

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

### Rocket files
You can configure additional local settings by creating a `Rocket.local.toml` file in the root directory.
This file will be loaded by Rocket if it exists.
It is included in the `.gitignore` file, so it won't be committed to the repository.

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

# Creating Users

To create a user without using OAUTH and without an SMTP server, you can use the following command:

```bash
curl -X POST -H "Content-Type: application/json" -d '{"email":"some@email", "password":"some_password"}' \
    http://localhost:8000/api/v1/signup
```

This will create a user with the given email and password in the database, and create a validation token.

In the terminal logs, you will see a message like this:
```
Url to validate email: http://localhost:8000/api/auth/verify?email=hans@larsen.online&token=pV1lD2qqKiJ0R7_AT1uPVnmeBjUbMjvOSfH8FI02wmw
```

Then use the link to validate the email address (no password necessary).
That should also set cookies and log you in.
