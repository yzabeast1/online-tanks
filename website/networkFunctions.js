function fetchWithFallback(url,headers) {

    return fetch(url, { headers })
        .then(response => {
            if (!response.ok) throw new Error('HTTPS failed');
            return response.json();
        })
        .catch(error => {
            console.error('HTTPS error:', error);
        });
}

function postWithFallback(url, headers) {

    // Try sending the message using HTTPS
    fetch(url, {
        method: 'POST',
        headers: headers
    })
        .then(response => {
            if (!response.ok) throw new Error('HTTPS failed'); // Handle HTTP errors
            return response.json();
        })
        .catch(error => {
            console.error('HTTPS error:', error);
        });
}
function postWithFallbackNoJSON(url, headers) {

    // Try sending the message using HTTPS
    fetch(url, {
        method: 'POST',
        headers: headers
    })
        .then(response => {
            if (!response.ok) throw new Error('HTTPS failed'); // Handle HTTP errors
            return response;
        })
        .catch(error => {
            console.error('HTTPS error:', error);
        });
}