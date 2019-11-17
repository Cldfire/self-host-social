<script>
    import { Link } from 'svero';

    import { signedIn } from './stores.js';

    let signedInValue;
    const unsubscribe = signedIn.subscribe(value => {
		signedInValue = value;
    });

    async function logOut() {
        const response = await fetch(
            "/api/logout",
            {
                method: 'POST',
                credentials: 'same-origin'
            }
        );
        if (response.status === 200) {
            signedIn.set(false);
        } else {
            // TODO: handle potential errors / issues
            // should reply with json payload
            alert("login failed");
        }
    }
</script>

{#if signedInValue}
    <p>Hi! You are signed in.</p>

    <button on:click="{logOut}">Log Out</button>
{:else}
    <p>You are not signed in.</p>

    <Link href="login">Log In</Link>
    <Link href="signup">Sign Up</Link>
{/if}
