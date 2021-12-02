<div align="center">
    <a href="#" target="_blank">
        <img src="https://raw.githubusercontent.com/appcypher/gigamono-assets/main/avatar-gigamono-boxed.png" alt="Gigamono Logo" width="140" height="140"></img>
    </a>
</div>

<h1 align="center">Engine Router</h1>

`engine-router` is a user-facing service. It's sole purpose is to provide TCP connections to clients and route requests and streams to the right subscribers.

> Information provided here is for folks working on this package. If your goal is to get started with the Gigamono framework, check the [Gigamono repo](https://github.com/gigamono/gigamono) on how to do that.


## Why I'm Deciding Against Using NATS for Ingress Traffic Load Balancing.

So I have decided to scrap engine-router and have engine-backend take http requests directly. I believe using a service mesh is better for ingress traffic and that NATS makes more sense for internal communication like engine-backend to engine-database.

Having engine-router serve as a pseudo-proxy that takes HTTP requests from the outside world and routes them via a NATS server to the right engine-backend is quite complicated and error-prone. NATS does not make a scenario like this easy to implement and the number of hops required makes me cringe. Sure service mesh has a similar number of hops but at least it is optimised for my use case.

As of writing, the following are the issues I faced with NATS:

1. NATS does not have a way to tell a client that there are no subscriber for the message it is sending. You could solve this by relying on timeouts (which is a haven for DOS attacks) or you could get the information from a NATS server which adds to latency. I also have to handle HTTP-related issues like retries. These are the kind of problems service mesh frameworks are designed to solve. They naturally support HTTP proxying out of the box, handling retries, acknowledgement, etc.

2. The significant latency from hopping between engine-router, NATS server and engine-backend is also an issue. With service mesh like linkerd, you get a more efficient proxy sidecar that reduces latency.

Implementing the request-response streaming not only made the engine-router more complex but also extended the complexity to engine-backend, which should solely be focused on orchestrating tera runtimes.

#### References

- [How I was going to solve the problem](https://gist.github.com/appcypher/dd806c20fe4872dae536539905cc8ccd)

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
  diesel migration run --database-url "postgresql://localhost:5432/routerdb"
  ```

- If you make changes to the migrations folder, update the `lib/db/schema.rs` file with:

  ```bash
  diesel print-schema --database-url "postgresql://localhost:5432/routerdb" > lib/db/schema.rs
  ```

- Updating the schema also means updating the models.

  ```bash
  diesel_ext > lib/db/models.rs
  ```
