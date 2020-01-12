import { readable } from 'svelte/store';

export const username = readable(getUsernameFromCookie());
export const loggedIn = readable(getUsernameFromCookie() !== "");

function getUsernameFromCookie() {
    let cookieName = "logged-in-user=";
    let cookie = decodeURIComponent(document.cookie)
        .split(";")
        .map(c => c.trim())
        .find(c => c.substr(0, cookieName.length) === cookieName)

    if (cookie === undefined) {
        return "";
    } else {
        return cookie.split("=")[1];
    }
}
