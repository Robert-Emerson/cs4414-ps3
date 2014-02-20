- Two processes running before the first request arrives. The first is the server process, the second listens for each request.

- The listen method spawns a task that waits for connections, and then when a connection is made, handles that connection in a new task.

- The request queue is FIFO, so any static requests will be served in the order they appear. Dyanmic requests are handled immediately.
