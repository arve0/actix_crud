<script>
    import Login from './Login.svelte';
    import CRUD from './CRUD.svelte';
    import { loggedIn, username } from './user.service.js';

    let showLogin = !$loggedIn;
    let showRegister = false;

    function showLoginDialog() {
        showRegister = false;
        showLogin = true;
    }

    function showRegisterDialog() {
        showLogin = false;
        showRegister = true;
    }
</script>

<header>
    <div class=brand>
        <a href="/">actix crud</a>
    </div>
    <div class=menu></div>
    <div class=login>
        {#if $loggedIn}
            <a class=button href="/user/logout">Logout ({$username})</a>
        {:else if showRegister}
            <button on:click={showLoginDialog}>Log in</button>
        {:else if showLogin}
            <button on:click={showRegisterDialog}>Register</button>
        {/if}
    </div>
</header>

<main>
    {#if $loggedIn}
        <CRUD />
    {:else if showLogin}
        <Login />
    {:else if showRegister}
        <Login type=register />
    {/if}
</main>

<style>
    header {
        background-color: aquamarine;
        padding: 1em;
        display: grid;
        grid-template-columns: max-content auto max-content;
    }
    header .brand {
        align-self: center;
    }
    header .login > * {
        margin: 0;
    }
    main {
        margin-top: 4em;
        margin-left: 0.5em;
        margin-right: 0.5em;
        margin-bottom: 1em;
    }
    @media screen and (min-width: 768px) {
        main {
            margin-left: 2em;
            margin-right: 2em;
        }
    }
</style>
