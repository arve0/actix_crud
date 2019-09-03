import { writable } from "svelte/store";

const BASE_URL = "/document"

// "pre-fetch"
const _initial_documents = fetch(BASE_URL).then(response => {
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
    update(documents => [...documents, { id, data }])
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
}

export const documents = { create, delete_, subscribe, put };
