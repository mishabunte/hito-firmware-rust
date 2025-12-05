#!/usr/bin/env python3
import socket
import time
import sys
import queue
import select

from PyQt6 import QtCore, QtWidgets

SOCKET_PATH = "/tmp/hito_Linux.sock"
BUFFER_SIZE = 4096
STELLAR_PREFIX = "stellar.sign:"
NETWORK_TO_HASH = {
    "Mainnet": "7ac33997544e3175d266bd022439b22cdb16508c01163f26e5cb2a3e1045a979",
    "Testnet": "cee0302d59844d32bdca915c8203dd44b33fbb7edc19051ea37abedf28ecd472",
    "Futurenet": "a3a1c6a78286713e29be0e9785670fa838d13917cd8eaeb4a3579ff1debc7fd5",
}
NETWORK_PREFIX = None

def create_test_message() -> bytes:
    prefix = STELLAR_PREFIX.encode("utf-8")
    # test payload: send 13 XLM to GCEMPQ7LYZ7EYWFYQYXIFQRUHKLRT74TLORALIOZIP2KPED7TD3GG352 with fee 5 XLM
    test_payload = b"AAAAAgAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAGQACs4eAAAAAwAAAAEAAAAAAAAAAAAAAABpLz/JAAAAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAAJiWgAAAAAAAAAAA"
    return prefix + NETWORK_PREFIX.encode("utf-8") + b":" + test_payload

class SocketThread(QtCore.QThread):
    log = QtCore.pyqtSignal(str)
    status_changed = QtCore.pyqtSignal(str)

    # GUI will emit this to push data to send
    enqueue_send = QtCore.pyqtSignal(bytes)

    def __init__(self, path: str, parent=None):
        super().__init__(parent)
        self.path = path
        self.sock: socket.socket | None = None
        self.running = False
        self.send_queue: queue.Queue[bytes] = queue.Queue()

        # This connects *inside* the thread context once it starts
        self.enqueue_send.connect(self._enqueue_send_impl)

    @QtCore.pyqtSlot(bytes)
    def _enqueue_send_impl(self, data: bytes):
        # Runs in socket thread, so queue is safe
        self.send_queue.put(data)

    def _connect_with_retry(self, retries: int = 5, delay: float = 0.25) -> bool:
        # Close previous socket if any
        if self.sock is not None:
            try:
                self.sock.close()
            except OSError:
                pass
            self.sock = None

        self.status_changed.emit("Connecting...")
        self.log.emit(f"Connecting to {self.path}...")

        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)

        for attempt in range(retries):
            try:
                client.connect(self.path)
                self.sock = client
                self.status_changed.emit("Connected")
                self.log.emit(f"[PY] Connected to {self.path}")
                return True
            except FileNotFoundError:
                self.log.emit(
                    f"[PY] Socket file not found (attempt {attempt + 1}/{retries}), retrying..."
                )
            except ConnectionRefusedError:
                self.log.emit(
                    f"[PY] Connection refused (attempt {attempt + 1}/{retries}), retrying..."
                )
            time.sleep(delay)

        self.log.emit(f"[PY] Failed to connect to {self.path}")
        self.status_changed.emit("Disconnected")
        self.sock = None
        return False

    def run(self):
        # This is the background IO loop
        if not self._connect_with_retry():
            return

        self.running = True
        self.log.emit("[PY] Socket thread started (recv + send loop)")

        try:
            while self.running and self.sock is not None:
                # 1) handle outgoing queue
                try:
                    while True:
                        data = self.send_queue.get_nowait()
                        try:
                            self.sock.sendall(data)
                            self.log.emit(f"[PY] Sent {len(data)} bytes")
                        except BrokenPipeError:
                            self.log.emit("[PY] Server closed the connection (Broken pipe).")
                            self.running = False
                            break
                        except OSError as e:
                            self.log.emit(f"[PY] Socket error while sending: {e}")
                            self.running = False
                            break
                except queue.Empty:
                    pass  # nothing to send right now

                if not self.running or self.sock is None:
                    break

                # 2) wait up to 100ms for incoming data
                rlist, _, _ = select.select([self.sock], [], [], 0.1)
                if rlist:
                    try:
                        data = self.sock.recv(BUFFER_SIZE)
                    except OSError as e:
                        self.log.emit(f"[PY] Socket error in recv: {e}")
                        self.running = False
                        break

                    if not data:
                        self.log.emit("[PY] No data (server may have closed connection).")
                        self.running = False
                        break

                    # Try UTF-8 decode, otherwise hex
                    try:
                        text = data.decode("utf-8")
                        self.log.emit(f"[PY] Received ({len(data)} bytes): {text!r}")
                    except UnicodeDecodeError:
                        self.log.emit(
                            f"[PY] Received ({len(data)} bytes, binary): {data.hex()}"
                        )

        finally:
            if self.sock is not None:
                try:
                    self.sock.close()
                except OSError:
                    pass
                self.sock = None
            self.status_changed.emit("Disconnected")
            self.log.emit("[PY] Socket thread finished")

    def stop(self):
        self.running = False
        # `run()` will exit after next select / loop iteration


class MainWindow(QtWidgets.QMainWindow):
    init_signal = QtCore.pyqtSignal()
    def __init__(self):
        super().__init__()

        self.setWindowTitle("Hito Communication Emulator")

        central = QtWidgets.QWidget()
        self.setCentralWidget(central)

        layout = QtWidgets.QVBoxLayout(central)

        # Status + connect
        top_layout = QtWidgets.QHBoxLayout()
        self.status_label = QtWidgets.QLabel("Disconnected")
        self.status_label.setStyleSheet("color: red;")
        top_layout.addWidget(self.status_label)

        # Message input
        header_layout = QtWidgets.QHBoxLayout()
        header_layout.addWidget(QtWidgets.QLabel("Message:"))

        # Clear message button
        clear_btn = QtWidgets.QPushButton("X")
        clear_btn.setStyleSheet("background-color: #ff0000; color: white; font-weight: bold;")
        header_layout.addWidget(clear_btn)
        clear_btn.clicked.connect(lambda: self.msg_edit.clear())
        clear_btn.setFixedWidth(40)
        layout.addLayout(header_layout)

        self.msg_edit = QtWidgets.QLineEdit()
        layout.addWidget(self.msg_edit)

        # Buttons for choosing network
        layout.addWidget(QtWidgets.QLabel("Choose Stellar Network:"))
        btn_layout = QtWidgets.QHBoxLayout()
        self.mainnet_btn = QtWidgets.QPushButton(f"Mainnet")
        self.testnet_btn = QtWidgets.QPushButton(f"Testnet")
        self.futurenet_btn = QtWidgets.QPushButton(f"Futurenet")
        btn_layout.addWidget(self.mainnet_btn)
        btn_layout.addWidget(self.testnet_btn)
        btn_layout.addWidget(self.futurenet_btn)
        layout.addLayout(btn_layout)
        self.network_buttons = {
            "Mainnet": self.mainnet_btn,
            "Testnet": self.testnet_btn,
            "Futurenet": self.futurenet_btn,
        }
        self.mainnet_btn.clicked.connect(lambda: self.on_set_network("Mainnet"))
        self.testnet_btn.clicked.connect(lambda: self.on_set_network("Testnet"))
        self.futurenet_btn.clicked.connect(lambda: self.on_set_network("Futurenet"))

        # Buttons for sending
        btn_layout = QtWidgets.QHBoxLayout()
        self.append_prefix_btn = QtWidgets.QPushButton(f"Append {STELLAR_PREFIX} prefix")
        self.send_stellar_btn = QtWidgets.QPushButton(f"Create {STELLAR_PREFIX} test message")
        btn_layout.addWidget(self.append_prefix_btn)
        btn_layout.addWidget(self.send_stellar_btn)
        layout.addLayout(btn_layout)

        self.send_custom_btn = QtWidgets.QPushButton("SEND MESSAGE")
        layout.addWidget(self.send_custom_btn)

        # Log output
        layout.addWidget(QtWidgets.QLabel("Log:"))
        self.log_edit = QtWidgets.QTextEdit()
        self.log_edit.setReadOnly(True)
        layout.addWidget(self.log_edit)

        # Socket thread (created but not started yet)
        self.socket_thread: SocketThread | None = None
        self.init_signal.connect(self.on_init)

        # GUI actions
        self.send_custom_btn.clicked.connect(self.on_send_custom)
        self.send_stellar_btn.clicked.connect(self.on_send_stellar)
        self.msg_edit.returnPressed.connect(self.on_send_custom)
        self.append_prefix_btn.clicked.connect(self.on_append_prefix)

        self.init_signal.emit()

    @QtCore.pyqtSlot()
    def on_init(self):
        self.on_set_network("Mainnet")  # Default
        self.on_connect()

    @QtCore.pyqtSlot()
    def on_connect(self):
        # If no thread or finished, create new one
        if self.socket_thread is None or not self.socket_thread.isRunning():
            self.socket_thread = SocketThread(SOCKET_PATH)
            self.socket_thread.log.connect(self.append_log)
            self.socket_thread.status_changed.connect(self.update_status)
            self.socket_thread.start()
        else:
            self.append_log("[PY] Socket thread already running")

    @QtCore.pyqtSlot()
    def on_send_custom(self):
        self.on_connect()  # Ensure connected
        if self.socket_thread is None or not self.socket_thread.isRunning():
            self.append_log("[PY] Not connected, cannot send.")
            return
        text = self.msg_edit.text()
        if not text:
            return
        data = text.encode("utf-8")
        self.socket_thread.enqueue_send.emit(data)
        self.msg_edit.clear()

    @QtCore.pyqtSlot()
    def on_send_stellar(self):
        self.msg_edit.setText(create_test_message().decode("utf-8"))

    @QtCore.pyqtSlot()
    def on_append_prefix(self):
        self.msg_edit.setText(STELLAR_PREFIX + self.msg_edit.text() + NETWORK_PREFIX + ":")

    @QtCore.pyqtSlot(str)
    def on_set_network(self, network: str):
        global NETWORK_PREFIX
        previous_network = NETWORK_PREFIX
        NETWORK_PREFIX = NETWORK_TO_HASH.get(network)
        self.append_log(f"[PY] Stellar network set to {network}")
        if previous_network is not None:
            self.msg_edit.setText(self.msg_edit.text().replace(previous_network, NETWORK_PREFIX))
        self.update_network_button_colors(network)

    @QtCore.pyqtSlot(str)
    def append_log(self, text: str):
        self.log_edit.append(text)

    def update_network_button_colors(self, selected: str):
        for name, btn in self.network_buttons.items():
            if name == selected:
                btn.setStyleSheet("background-color: #a9a9a9; color: white; font-weight: bold; border: 1px solid black;")
            else:
                btn.setStyleSheet("")

    @QtCore.pyqtSlot(str)
    def update_status(self, status: str):
        self.status_label.setText(status)
        if status == "Connected":
            self.status_label.setStyleSheet("color: green;")
        elif status == "Connecting...":
            self.status_label.setStyleSheet("color: orange;")
        else:
            self.status_label.setStyleSheet("color: red;")

    def closeEvent(self, event):
        if self.socket_thread is not None and self.socket_thread.isRunning():
            self.socket_thread.stop()
            self.socket_thread.wait(1000)
        super().closeEvent(event)


def main():
    app = QtWidgets.QApplication(sys.argv)
    win = MainWindow()
    win.resize(700, 500)
    win.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
