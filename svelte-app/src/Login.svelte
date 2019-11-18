<script>
    import { navigateTo } from 'svero';

    import { signedIn, userId } from './stores.js';

    async function handleSubmit(event) {
        if(!event.target.checkValidity()) {
            return;
        }

        const response = await fetch(
            "/api/login",
            {
                method: 'POST',
                headers: {
                    'Accept': 'application/json',
                    'Content-Type': 'application/json'
                },
                credentials: 'same-origin',
                body: JSON.stringify({
                    email: event.target.email.value,
                    password: event.target.password.value,
                })
            }
        );
        const userInfo = await response.json();

        if (response.status === 202) {
            signedIn.set(true);
            userId.set(userInfo.user_id);
            navigateTo('/')
        } else {
            // TODO: handle potential errors / issues
            // should reply with json payload
            alert("login failed");
        }
    }
</script>

<form on:submit|preventDefault="{handleSubmit}">
    <label for="email">Email</label>
    <input required type="email" id="email"/>

    <label for="password">Password</label>
    <input required type="password" id="password"/>

    <button type="submit">Log In</button>
</form>
