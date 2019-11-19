const fetch = require("node-fetch")
const assert = require("assert")

describe("documents sorted by date", function () {
    before(async function () {
        await register_and_login()

        let creating_documents = [...Array(123).keys()].map(create_document);
        await Promise.all(creating_documents);
    })

    it("should get 100 first documents", async function () {
        let documents = await get_documents()
        assert.equal(documents.length, 100)
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

function get_documents() {
    return fetch(BASE_URL + "/document", { headers: { cookie } })
        .then(r => r.json())
}