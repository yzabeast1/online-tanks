// document.getElementById('start-chat-button').addEventListener('click', startChat);
document.getElementById('send-button').addEventListener('click', sendMessage);
document.getElementById('message-input').addEventListener('keydown', handleKeyPress);
// document.getElementById('start-chat-button').addEventListener('click', startChat);
var chatCooldown=1000;
var chatInterval=0;
function startChat() {
    // Get the joincode and username from input fields
    joincode = document.getElementById('joincode-input').value;
    username = document.getElementById('username-input').value;

    if (joincode.trim() === '' || username.trim() === '') {
        alert('Please enter both a join code and username');
        return;
    }

    // Hide login form and show the chat box
    // document.querySelector('.menu-screen').style.display = 'none';
    // document.querySelector('.chat-container').style.display = 'flex';

    // Start fetching chat messages periodically
    chatInterval=setInterval(function() {fetchChatMessages(joincode)}, chatCooldown);
    fetchChatMessages(joincode);
}

function sendMessage() {
    const input = document.getElementById('message-input');
    const message = input.value;

    if (message.trim() === "") return; // Prevent sending empty messages

    // Add the message to the chat box locally
    addMessageToChatBox(`${username}: ${message}`, 'user-message');
    const chatBox = document.getElementById('chat-box');
    chatBox.scrollTop = chatBox.scrollHeight;
    const headers = {
        'Content-Type': 'application/json',
        'joincode': joincode,  // Use dynamic joincode
        'username': username,  // Use dynamic username
        'text': message,
        ...shooHeaders(shooAccountDetails()),
    };

    // Send the message to the server with headers
    postWithFallbackNoJSON(`https://${serverip}/sendChat`, headers);

    // Clear the input box after sending the message
    input.value = '';
}

// Handle "Enter" key press to send message
function handleKeyPress(event) {
    if (event.key === 'Enter') {
        sendMessage(); // Send the message when "Enter" is pressed
        event.preventDefault(); // Prevent the default behavior of the Enter key (e.g., new line)
    }
}

function fetchChatMessages(joincode) {
    const headers = { 'joincode': joincode }; // Add joincode header
    fetchWithFallback(`https://${serverip}/getChat`,headers)
        .then(data => {
            if (data && data.length > 0) {
                const chatBox = document.getElementById('chat-box');
                const distanceFromBottom = chatBox.scrollHeight - chatBox.scrollTop - chatBox.clientHeight;
                const shouldStickToBottom = distanceFromBottom < 24;
                const previousScrollTop = chatBox.scrollTop;
                clearChatBox();
                data.forEach(message => {
                    if(message.sender!='server')addMessageToChatBox(`${message.sender}: ${message.message}`, message.sender === username ? 'user-message' : 'server-message');
                    else addLogToChatBox(`${message.message}`,'action-log')
                });
                if (shouldStickToBottom) {
                    chatBox.scrollTop = chatBox.scrollHeight;
                } else {
                    chatBox.scrollTop = previousScrollTop;
                }
            } else {
                console.warn('No messages found for this joincode.');
            }
        })
        .catch(error => {
            console.error('Error fetching chat messages:', error);
        });
}

// Helper function to append a message to the chat box
function addMessageToChatBox(message, className) {
    const messageElement = document.createElement('div');
    messageElement.className = className;
    messageElement.innerText = message;

    const chatBox = document.getElementById('chat-box');
    chatBox.appendChild(messageElement);
}
function addLogToChatBox(message, classname){
    const messageElement = document.createElement('div');
    messageElement.className = classname;
    const strong=document.createElement('strong')
    strong.innerText="Action: "+message
    messageElement.appendChild(strong)
    const chatBox = document.getElementById('chat-box');
    chatBox.appendChild(messageElement);
}

function clearChatBox() {
    const chatBox = document.getElementById('chat-box');
    chatBox.innerHTML = ''; // Clear all messages
}
