<script>
    import { Router, Route } from 'svero';
    import { onMount } from 'svelte';

    import Index from './Index.svelte';
    import Signup from './Signup.svelte';
    import Login from './Login.svelte';
    import { signedIn } from './stores.js';

    onMount(async () => {
        const response = await fetch(
            "/api/me",
            {
                method: 'POST',
                credentials: 'same-origin'
            }
        );

        if (response.status === 200) {
            signedIn.set(true);
        }
    });
</script>

<Router>
    <Route exact path="/" component={Index}/>
    <Route path="/signup" component={Signup}/>
    <Route path="/login" component={Login}/>
    <!-- TODO: Figure out how to set up a wildcard path. Right now entering a
    non-existent path results in a network request, but that shouldn't occur -->
</Router>
