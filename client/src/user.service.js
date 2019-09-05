import { readable } from 'svelte/store';

// "pre-fetch"
const _username = fetch('/user').then(r => {
    if (r.status !== 200) {
        return null;
    }
    return r.text();
});

export const username = readable(null, set => {
    _username.then(set);
});

export const loggedIn = readable(false, set => {
    _username.then(u => u !== null).then(set);
});

export const loggedInPromise = _username.then(u => u !== null);
