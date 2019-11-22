<script>
    import { Link } from 'svero';

    import Post from './Post.svelte';

    import { signedIn, userId } from './stores.js';
    import { loadRecentPosts } from './utils.js'

    // TODO: this attempts to load posts even when not signed in
    var recentPostsPromise = loadRecentPosts(null, 15);

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
    <Link href="new-post">New Post</Link>
    <button on:click="{logOut}">Log Out</button>

    <h2>Recent Posts</h2>
    {#await recentPostsPromise then recentPosts}
        {#each recentPosts as post}
            <Post postInfo={post}/>
        {/each}
    {/await}
{:else}
    <p>You are not signed in.</p>

    <Link href="login">Log In</Link>
    <Link href="signup">Sign Up</Link>
{/if}
