## Engine API

The `engine-api` is the front-facing service of the Gigamono framework that the part users interact with.

> Information provided here is for folks working on this package. If your goal is to get started with the Gigamono framework, check the [Gigamono repo](https://github.com/gigamono/gigamono) on how to do that.

##

### Content

1. [Setting up with Docker](#docker-start)

2. [Setting up without Docker](#no-docker-start)

3. [Managing the Database](#managing-db)

##

### Setting up with Docker <a name="docker-start" />

- It is easy.

  ```bash
  GIGAMONO_CONFIG_PATH="[path/to/gigamono.yaml]" docker-compose -f docker/compose.yaml up
  ```

  This automatically creates and starts a nats server and a postgres db you can work with.

##

### Setting up without Docker <a name="no-docker-start" />

If you prefer to set things up yourself, you can follow this guide.

- Check [managing the database](#managing-db) to prepare a database for the project.

- You will also need a nats server running. Check [here](https://docs.nats.io/nats-server/installation) for more information on how to do that.

- Set `GIGAMONO_CONFIG_PATH` variable to the location of your Gigamono config file.

  ```bash
  export GIGAMONO_CONFIG_PATH="[path/to/gigamono.yaml]"
  ```

    <details><summary>other shells</summary>

  ##### Fish

  ```fish
  set -x GIGAMONO_CONFIG_PATH "[path/to/gigamono.yaml]"
  ```

    </details>

- Start the server

  ```bash
  RUST_LOG=info cargo r
  ```

##

### Managing the Database <a name="managing-db" />

- Create a postgres database if you don't already have one.

  If you are developing with the docker environment, this should already be set up for you.

- Install `diesel_cli` and `diesel_cli_ext`

  ```bash
  cargo install diesel_cli --no-default-features --features postgres
  ```

  ```bash
  cargo install diesel_cli_ext
  ```

- Run migrations

  ```bash
  diesel migration run --database-url "postgresql://localhost:5040/apidb"
  ```

- If you make changes to the migrations folder, update the `lib/db/schema.rs` file with:

  ```bash
  diesel print-schema --database-url "postgresql://localhost:5040/apidb" > lib/db/schema.rs
  ```

- Updating the schema also means updating the models.

  ```bash
  diesel_ext > lib/db/models.rs
  ```
