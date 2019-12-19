
jQuery( document ).ready(function() {
    console.log("Hello");
    let socket = new WebSocket('ws://' + location.host + '/v1/connect');
    socket.onopen = function() {
        console.log("Socket opened.")
    }
    socket.onmessage = function(event) {
      let message = event.data;
      console.log(message);
    };
});