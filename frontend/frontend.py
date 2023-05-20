import http.server
import os

class MyRequestHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        file_extension = os.path.splitext(self.path)[1]

        if self.path.startswith('/') and not file_extension:
            self.path = 'index.html'
        return super().do_GET()

http.server.test(HandlerClass=MyRequestHandler)
