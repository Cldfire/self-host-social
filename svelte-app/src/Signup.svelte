<script>
    import { navigateTo } from 'svero';

    import { signedIn, userId } from './stores.js';

    async function handleSubmit(event) {
        if(!event.target.checkValidity()) {
            return;
        }

        const response = await fetch(
            "/api/signup",
            {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                credentials: 'same-origin',
                body: JSON.stringify({
                    email: event.target.email.value,
                    password: event.target.password.value,
                    display_name: event.target.displayName.value,
                    real_name: event.target.realName.value
                })
            }
        );

        if (response.ok) {
            const userInfo = await response.json();

            signedIn.set(true);
            userId.set(userInfo.user_id);
            navigateTo('/')
        } else {
            // TODO: handle potential errors / issues
            // should reply with json payload
            alert("signup failed");
        }
    }
</script>

<form on:submit|preventDefault="{handleSubmit}">
    <label for="email">Email</label>
    <input required type="email" id="email"/>

    <label for="password">Password</label>
    <input required type="password" id="password"/>

    <label for="displayName">Display Name</label>
    <input required id="displayName"/>

    <label for="realName">Real Name</label>
    <input required id="realName"/>

    <button type="submit">Create account</button>
</form>
