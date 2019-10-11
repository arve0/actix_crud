import { writable } from "svelte/store";
import { loggedInPromise } from './user.service.js'

const BASE_URL = "/document"

// "pre-fetch"
const _initial_documents = loggedInPromise.then(isLoggedIn => {
    if (isLoggedIn) {
        return fetch(BASE_URL);
    }
    return {} // empty response
}).then(response => {
    if (response.status !== 200) {
        return [];
    }
    return response.json();
});

const { set, subscribe, update } = writable([], set => _initial_documents.then(set));

async function create(data) {
    let response = await fetch(BASE_URL, {
        method: 'POST',
        headers: { "content-type": "application/json" },
        body: JSON.stringify(data)
    })

    if (response.status !== 201) {
        let msg = await response.text();
        throw new Error(`HTTP status ${response.status}, message '${msg}'`);
    }
    let id = await response.text();
    let document = { id, data };
    update(documents => [...documents, document])
    // communicate with tabs
    localStorage.setItem("insert", JSON.stringify(document))
}

async function delete_(id) {
    let response = await fetch(`${BASE_URL}/${id}`, {
        method: "DELETE"
    })
    if (response.status !== 200) {
        let msg = await response.text();
        throw new Error(`HTTP status ${response.status}, message '${msg}'`);
    }
    update(documents => {
        const i = documents.findIndex(o => o.id === id)

        if (i === -1) {
            throw new Error(`Expected to find id ${id}, but did not.`)
        }

        return [
            ...documents.slice(0, i),
            ...documents.slice(i + 1)
        ]
    })
    // communicate with tabs
    localStorage.setItem("delete", id)
}

async function put(document) {
    let { id, data } = document;

    let response = await fetch(`${BASE_URL}/${id}`, {
        method: 'PUT',
        headers: { "content-type": "application/json" },
        body: JSON.stringify(data)
    })

    if (response.status !== 200) {
        let msg = await response.text();
        throw new Error(`HTTP status ${response.status}, message '${msg}'`);
    }
    update(documents => {
        const i = documents.findIndex(o => o.id === id)

        if (i === -1) {
          throw new Error(`Expected to find id ${id}, but did not.`)
        }

        return [
            ...documents.slice(0, i),
            document,
            ...documents.slice(i + 1)
        ]
    })
    // communicate with tabs
    localStorage.setItem("update", JSON.stringify(document))
}

const updates = new EventSource("/updates")
updates.addEventListener("insert", insertFromUpdates);
updates.addEventListener("update", putFromUpdates);
updates.addEventListener("delete", deleteFromUpdates);

window.addEventListener('storage', function (event) {
    if (event.key === "insert") {
        insertFromUpdates({ data: event.newValue })
    } else if (event.key === "update") {
        putFromUpdates({ data: event.newValue })
    } else if (event.key === "delete") {
        deleteFromUpdates({ data: event.newValue })
    }
});

function insertFromUpdates(event) {
    let document = JSON.parse(event.data);
    update(documents => [...documents, document]);
}

function putFromUpdates(event) {
    let document = JSON.parse(event.data);
    update(documents => documents.map(d =>
        d.id === document.id
            ? document
            : d
    ))
}

function deleteFromUpdates(event) {
    let id = event.data;
    update(documents => documents.filter(d => d.id !== id))
}

export const documents = { create, delete_, subscribe, put };
