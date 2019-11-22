// Returns details about n recent posts for the given user id.
//
// If a user id is not provided it returns details about n recent posts for all
// users globally.
async function loadRecentPosts(user_id, n) {
	var response;
	const params = {
		method: 'GET',
		credentials: 'same-origin'
	}

	if (user_id === null) {
		response = await fetch(
			"/api/recent-posts?n=" + n,
			params
		);
	} else {
		response = await fetch(
			"/api/recent-posts?req_user_id=" + user_id + "&n=" + n,
			params
		);
	}

	if (response.ok) {
		return response.json();
	} else {
		// TODO:
		return {};
	}
}

export { loadRecentPosts };
