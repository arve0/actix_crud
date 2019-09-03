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

<header class=navbar>
    <div class=navbar-brand>
        <span class=navbar-item>actix crud</span>
    </div>
    <div class=navbar-end>
        <div class=navbar-item>
            {#if $loggedIn}
                <a href="/user/logout" class=button>Logout ({$username})</a>
            {:else}
                <button class=button on:click={showLoginDialog}>Log in</button>
                <button class=button on:click={showRegisterDialog}>Register</button>
            {/if}
        </div>
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
        /* padding: 1em; */
    }
    main {
        margin-top: 2em;
    }
</style>
