from PySide6.QtCore import SIGNAL, QObject, QUrl

from PySide6.QtWidgets import QApplication, QInputDialog, QLineEdit, QMainWindow, QMessageBox
from ui_main_window import Ui_MainWindow

import urllib.parse

from leda import gemini

class MainWindow(QMainWindow):
    def __init__(self):
        super(MainWindow, self).__init__()
        self.ui = Ui_MainWindow()
        self.ui.setupUi(self)

        self.client = gemini.Client()
        self.client.set_timeout(5)

        self.current_page = "gemini://gemini.circumlunar.space/"
        self.history = []
        self.history_index = -1

        self.ui.back_button.clicked.connect(self.__handle_back)
        self.ui.forward_button.clicked.connect(self.__handle_forward)
        self.ui.refresh_button.clicked.connect(lambda: self.__get_page(self.history[self.history_index]))
        self.ui.url_input.returnPressed.connect(self.__handle_url_update)

        self.ui.content.setOpenLinks(False)
        self.ui.content.anchorClicked.connect(self.__handle_link)

        self.ui.url_input.setText(self.current_page)
        self.__push_to_history(self.current_page)
        self.__get_page(self.current_page)

    def __get_page(self, url: str):
        """
        Attemps to load the page address in url, the url doesn't need to include
        the scheme, it'll be assumed gemini by default. Popping
        up an error message to the user if issues occured.
        """

        if not url.startswith("gemini://"):
            url = "gemini://" + url

        response = ()
        status = ()
        meta = ()

        try:
            response = self.client.request(url)
            (status, meta) = response.header
        except Exception as err:
            QMessageBox.warning(self, "Failed to acquire resource", str(err))
            self.ui.url_input.setText(self.current_page)
            return

        match status:
            case 10 | 11:
                self.__handle_input(status, meta)
            case 20:
                self.__handle_success(meta, response.body)
            case 30 | 31:
                self.__handle_redirect(status, meta)
            case 40 | 41 | 42 | 43 | 44:
                self.__handle_temp_fail(status, meta)
            case 50 | 51 | 52 | 53 | 59:
                self.__handle_perma_fail(status, meta)
            case 60 | 61 | 62:
                self.__handle_cert_fail(status, meta)

        enable_back = False if self.history_index == 0 else True
        enable_forward = False if self.history_index == len(self.history) - 1 else True

        self.ui.back_button.setEnabled(enable_back)
        self.ui.forward_button.setEnabled(enable_forward)
        self.ui.url_input.setText(url)

    def __handle_back(self):
        self.history_index -= 1
        self.__get_page(self.history[self.history_index])

    def __handle_forward(self):
        self.history_index += 1
        self.__get_page(self.history[self.history_index])

    # call this function to properly update the history list
    def __push_to_history(self, link: str):
        if self.history_index == len(self.history) - 1:
            self.history.append(link)
            self.history_index += 1
        else:
            del self.history[self.history_index+1:]
            self.history.append(link)
            self.history_index += 1

    # handle the signal coming from user pressing enter on the url bar
    def __handle_url_update(self):
        self.__push_to_history(self.ui.url_input.text())
        self.__get_page(self.ui.url_input.text())

    def __handle_link(self, url: QUrl):
        target_url = ()
        if url.isRelative():
            base_url = QUrl(self.ui.url_input.text())
            target_url = base_url.resolved(url).toString()
        else:
            target_url = url.toString()

        self.__push_to_history(target_url)
        self.__get_page(target_url)

    def __handle_input(self, status: str, meta: str):
        echo_mode = QLineEdit.EchoMode.Normal if status == 10 else QLineEdit.EchoMode.Password

        text, ok = QInputDialog.getText(self, "Server input request",
            "Input request from gemini server\n" + meta, echo_mode)

        if ok and text:
            request = self.ui.url_input.text() + "?" + urllib.parse.quote(text)
            self.__push_to_history(request)
            self.__get_page(request)

    def __handle_success(self, meta: str, body):
        if meta == "":
            meta = "text/gemini; charset=utf-8"

        content = ()

        if meta.startswith("text/plaintext"):
            content = bytes(body).decode("utf-8")
        elif meta.startswith("text/gemini"):
            content = gemini.Gemtext.to_html(bytes(body).decode("utf-8"))

        self.ui.content.setText(content)

    def __handle_redirect(self, status: str, meta: str):
        user_confirmation = ()
        # automatically accept simple redirects
        if meta == self.ui.url_input.text() + "/" and self.ui.url_input.text()[-1] != "/":
            user_confirmation = True
        else:
            type = "temporarily"
            if status == 31:
                type = "permanently"

            user_confirmation = QMessageBox.question(self, "Redirect notice", "This resource " + type +
                " redirects to \"" + meta + "\". Would you like to proceed?")

        if user_confirmation:
            self.__push_to_history(meta)
            self.__get_page(meta)

    def __handle_temp_fail(self, status: str, meta: str):
        title = ()
        body = ()

        match status:
            case 40:
                title = "Temporary failure"
                body = "The server is facing a temporary failure. Additional info: " + meta
            case 41:
                title = "Server unavailable"
                body = "The server is unavailable due to overload or maintenance. Additional \
                    info: " + meta
            case 42:
                title = "CGI error"
                body = "A CGI process, died unexpectedly or timed out. Additional info: " + meta
            case 43:
                title = "Proxy error"
                body = "A proxy request  failed becase the server was unable to successfully \
                    complete a transaction with the remote host. Additional info: " + meta
            case 44:
                title = "Slow down"
                body = "Rate limiting is in effect, must wait " + meta + " seconds before sending \
                    another request."

        QMessageBox.warning(self, title, body)
        self.ui.url_input.setText(self.current_page)

    def __handle_perma_fail(self, status: str, meta: str):
        title = ()
        body = ()

        match status:
            case 50:
                title = "Permanent failure"
                body = "The server is facing a permanent failure. Additional info: " + meta
            case 51:
                title = "Not found"
                body = "The requested resource could not be found but may be available in the \
                    future. Additional info: " + meta
            case 52:
                title = "Gone"
                body = "The resource requested is no longer available and will not be available \
                    again. Additional info: " + meta
            case 53:
                title = "Proxy request refused"
                body = "The request was for a resource at a domain not served by the server and \
                    the server does not accept proxy requests. Additional info: " + meta
            case 59:
                title = "Bad request"
                body = "The server was unable to parse the client's request. Additional \
                    info: " + meta

        QMessageBox.warning(self, title, body)
        self.ui.url_input.setText(self.current_page)

    def __handle_cert_fail(self, status: str, meta: str):
        title = ()
        body = ()

        match status:
            case 60:
                title = "Client certificate required"
                body = "A client certificate is required for this this request. Additional info: "\
                    + meta
            case 61:
                title = "Certificate not authorized"
                body = "The supplied client certificate is not authorised for accessing the \
                    particular requested resource. The problem is not with the certificate \
                    itself, which may be authorised for other resources. Additional info: " + meta
            case 62:
                title = "Certificate not valid"
                body = "The supplied client certificate was not accepted because it is not valid. \
                    This indicates a problem with the certificate in and of itself. Additional \
                    info: " + meta

        QMessageBox.warning(self, title, body)
        self.ui.url_input.setText(self.current_page)