<script>
    import { navigateTo } from 'svero';

    import { userId } from './stores.js';
    // TODO: replace with official get fn when that is available
    import { get_store_value } from 'svelte/internal'

    async function handleSubmit(event) {
        if(!event.target.checkValidity()) {
            return;
        }

        const response = await fetch(
            "/api/create-post",
            {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                credentials: 'same-origin',
                body: JSON.stringify({
                    body: event.target.body.value
                })
            }
        );

        if (response.ok) {
            const creationResponse = await response.json();
            const arrayBuf = await event.target.file.files[0].arrayBuffer();

            const imgResponse = await fetch(
                "/api/set-post-image/" + creationResponse.post_id,
                {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/octet-stream'
                    },
                    credentials: 'same-origin',
                    body: arrayBuf
                }
            );

            if (!imgResponse.ok) {
                alert("image upload failed");
            } else {
                navigateTo("/user/" + get_store_value(userId));
            }
        } else {
            // TODO: handle potential errors / issues
            // should reply with json payload
            alert("creation failed");
        }
    }

    function previewFile() {
        var preview = document.getElementById("preview");
        var file = document.getElementById("file").files[0];
        var reader = new FileReader();

        reader.addEventListener("load", function () {
            preview.src = reader.result;
        }, false);

        if (file) {
            reader.readAsDataURL(file);
        }
    }
</script>

<form on:submit|preventDefault="{handleSubmit}">
    <label for="body">Post Description</label>
    <textarea id="body"></textarea>

    <label for="file">Choose Image</label>
    <input id="file" type="file" on:change="{previewFile}" accept="image/*">

    <button type="submit">Create post</button>
</form>

<img id="preview" src="" width="300" alt="Image preview...">
