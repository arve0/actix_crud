const fetch = require("node-fetch")
const assert = require("assert")

const CREATE_N_DOCUMENTS = 123;

describe("documents", function () {
    before(async function () {
        await register_and_login()

        let creating_documents = [...Array(CREATE_N_DOCUMENTS).keys()].map(create_document);
        await Promise.all(creating_documents);
    })

    it("should get 100 first documents", async function () {
        let documents = await get_documents()
        assert.equal(documents.length, 100)
    })

    it("should be sorted by primary key, latest insertion first", async function () {
        let documents = await get_documents()

        assert.equal(documents[0].pk, CREATE_N_DOCUMENTS);

        let prev = documents[1];
        for (let document of documents.slice(1)) {
            assert(document.pk <= prev.pk);
            prev = document;
        }
    })

    it("should get below primary key", async function () {
        let documents = await get_documents({ below_pk: 51 })

        assert.equal(documents.length, 50); // sqlite is 1-indexed

        let pk = 50;
        for (let document of documents) {
            assert.equal(document.pk, pk);
            pk -= 1;
        }
    })

    it("should create documents idempotent", async function () {
        let a = await create_document_with_id_and_data("asdf")
        let b = await create_document_with_id_and_data("asdf")

        assert.deepEqual(a, b);
    })

    it("should not allow creating with same id when data differs", async function () {
        let a = await create_document_with_id_and_data("different-data", 1234)
        try {
            let b = await create_document_with_id_and_data("different-data", 4321)
        } catch (error) {
            return
        }
        throw new Error("Should not be able to create document twice with different data")
    })

    it("should have a header with link to next page", async function () {
        let response = await get_documents_response()
        let next_link = response.headers.get("link-next")
        let documents = await response.json()
        let last_pk = documents[documents.length - 1].pk;
        let expected_link = `/document?below_pk=${last_pk}`

        assert.equal(next_link, expected_link)
    })

    it("should not link to next page when last page", async function () {
        let response = await get_documents_response({ below_pk: 100 })
        let next_link = response.headers.get("link-next")
        assert.equal(next_link, null)
    })

    it("should have a header with link to prev page", async function () {
        let response = await get_documents_response({ below_pk: 100 })
        let prev_link = response.headers.get("link-prev")
        let documents = await response.json()
        let first_pk = documents[0].pk;
        let expected_link = `/document?above_pk=${first_pk}`

        assert.equal(prev_link, expected_link)
    })

    it("should not link to prev page when first page", async function () {
        let response = await get_documents_response()
        let prev_link = response.headers.get("link-prev")
        assert.equal(prev_link, null)
    })
})

const BASE_URL = "http://localhost:8080"
let cookie = ""
const json = { "content-type": "application/json" }

async function register_and_login() {
    const register_result = await fetch(BASE_URL + "/user/register", {
        method: "POST",
        headers: { "content-type": "application/x-www-form-urlencoded" },
        body: "username=mocha&password=mocha",
        redirect: "manual",
    })

    const status = register_result.status
    const reason = await register_result.text()
    if (status !== 303 && reason !== "user registered") {
        throw new Error(`Unable to register and login, got ${status}: ${reason}`)
    }

    cookie = register_result.headers
        .get("set-cookie")
        .toString()
        .split(";")[0];

    if (!cookie) {
        throw new Error("Unable to register and login, got empty cookie")
    }
}

async function create_document(i) {
    let result = await fetch(BASE_URL + "/document", {
        method: "POST",
        headers: { cookie, ...json },
        body: `{"n":${i}}`,
    });

    if (result.status !== 201) {
        let reason = await result.text();
        console.error(result.status, reason);
    }
}

async function create_document_with_id_and_data(id, data="data") {
    let result = await fetch(BASE_URL + "/document/" + id, {
        method: "POST",
        headers: { cookie, ...json },
        body: `{"key":${JSON.stringify(data)}}`,
    });

    if (result.status !== 201) {
        let reason = await result.text();
        throw new Error(`Status: ${result.status}, body: ${reason}`)
    }

    return await result.json()
}

function get_documents({ below_pk } = {}) {
    return get_documents_response({ below_pk })
        .then(r => r.json())
}

function get_documents_response({ below_pk } = {}) {
    let below = below_pk !== undefined
        ? `?below_pk=${below_pk}`
        : "";
    return fetch(BASE_URL + `/document${below}`, { headers: { cookie } })
}