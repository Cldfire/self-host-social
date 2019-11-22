<script>
    import { Router, Route } from 'svero';
    import { onMount } from 'svelte';

    import Index from './Index.svelte';
    import Signup from './Signup.svelte';
    import Login from './Login.svelte';
    import UserProfile from './UserProfile.svelte';
    import NewPost from './NewPost.svelte';
    import { signedIn, userId } from './stores.js';

    onMount(async () => {
        const response = await fetch(
            "/api/me",
            {
                method: 'POST',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
            const userInfo = await response.json();

            signedIn.set(true);
            userId.set(userInfo.user_id);
        }
    });
</script>

<Router>
    <Route exact path="/" component={Index}/>
    <Route path="/signup" component={Signup}/>
    <Route path="/login" component={Login}/>
    <Route path="/new-post" component={NewPost}/>
    <Route path="/user/:userId" component={UserProfile}/>
    <!-- TODO: Figure out how to set up a wildcard path. Right now entering a
    non-existent path results in a network request, but that shouldn't occur -->
</Router>
