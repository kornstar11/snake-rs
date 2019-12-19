
jQuery( document ).ready(function() {
    let socket = new WebSocket("wss://:3031/v1/connect");
    socket.onmessage = function(event) {
      let message = event.data;
      console.log(message);
    };
});