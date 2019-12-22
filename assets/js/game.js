
jQuery( document ).ready(function() {
    console.log("Hello");
    //{"my_snake":[0,{"direction":"Right","length":3,"points":[{"x":70,"y":10},{"x":69,"y":10},{"x":68,"y":10}]}],"other_snakes":[[1,{"direction":"Right","length":3,"points":[{"x":45,"y":10},{"x":44,"y":10},{"x":43,"y":10}]}]]}
    let canvas = document.getElementById('canvas');
    let ctx = canvas.getContext('2d');
    let socket = new WebSocket('ws://' + location.host + '/v1/connect');

    socket.onopen = function() {
        console.log("Socket opened.")
    }

    socket.onmessage = function(event) {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      let message = event.data;
      let obj = JSON.parse(message);
      let my_snake = obj["my_snake"][1];
      let other_snakes = obj["other_snakes"];
      let points = my_snake["points"];
      render_snake(points);

      for(i = 0; i < other_snakes.length; i++) {
        let other_snake = other_snakes[i][1];
        render_snake(other_snake['points']);
      }
    };

    function render_snake(points) {
        ctx.beginPath();
        let first = true;
        for (const idx in points) {
          let point = points[idx];
          if(first == true) {
              ctx.moveTo(point["x"], point["y"]);
              first = false;
          } else {
              ctx.lineTo(point["x"], point["y"]);
          }
        }
        ctx.stroke();
    }

    document.addEventListener('keydown', (event) => {
        console.log(event)
        let keyCode = event['code'];
        let toSend = null;
        if(keyCode === 'ArrowUp') {
            toSend = 'Up';
        } else if (keyCode === 'ArrowDown') {
            toSend = 'Down';
        } else if (keyCode === 'ArrowLeft') {
            toSend = 'Left';
        } else if (keyCode === 'ArrowRight') {
            toSend = 'Right';
        }

        if(toSend != null) {
            socket.send(toSend);
        }
    })
});