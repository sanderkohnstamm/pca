import gi
gi.require_version('Gst', '1.0')
gi.require_version('GstRtspServer', '1.0')
from gi.repository import Gst, GstRtspServer, GObject

class RTSPServer:
    def __init__(self):
        self.server = GstRtspServer.RTSPServer()
        self.factory = GstRtspServer.RTSPMediaFactory()
        self.factory.set_launch(
            "( avfvideosrc ! videoconvert ! x264enc tune=zerolatency bitrate=500 speed-preset=superfast ! "
            "rtph264pay config-interval=1 name=pay0 pt=96 )"
        )
        self.factory.set_shared(True)
        self.server.get_mount_points().add_factory("/test", self.factory)
        self.server.attach(None)

    def run(self):
        loop = GObject.MainLoop()
        print("Stream ready at rtsp://127.0.0.1:8554/test")
        loop.run()

if __name__ == "__main__":
    Gst.init(None)
    server = RTSPServer()
    server.run()