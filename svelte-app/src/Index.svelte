<script>
    import { Link } from 'svero';

    import { signedIn, userId } from './stores.js';

    async function logOut() {
        const response = await fetch(
            "/api/logout",
            {
                method: 'POST',
                credentials: 'same-origin'
            }
        );
        if (response.ok) {
            signedIn.set(false);
            userId.set(-1);
        } else {
            // TODO: handle potential errors / issues
            // should reply with json payload
            alert("login failed");
        }
    }
</script>

{#if $signedIn}
    <p>Hi! You are signed in.</p>

    <Link href="user/{$userId}">View Profile</Link>
    <button on:click="{logOut}">Log Out</button>
{:else}
    <p>You are not signed in.</p>

    <Link href="login">Log In</Link>
    <Link href="signup">Sign Up</Link>
{/if}
