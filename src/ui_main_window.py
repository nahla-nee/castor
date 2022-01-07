# -*- coding: utf-8 -*-

################################################################################
## Form generated from reading UI file 'main_window.ui'
##
## Created by: Qt User Interface Compiler version 6.2.2
##
## WARNING! All changes made in this file will be lost when recompiling UI file!
################################################################################

from PySide6.QtCore import (QCoreApplication, QDate, QDateTime, QLocale,
    QMetaObject, QObject, QPoint, QRect,
    QSize, QTime, QUrl, Qt)
from PySide6.QtGui import (QBrush, QColor, QConicalGradient, QCursor,
    QFont, QFontDatabase, QGradient, QIcon,
    QImage, QKeySequence, QLinearGradient, QPainter,
    QPalette, QPixmap, QRadialGradient, QTransform)
from PySide6.QtWidgets import (QApplication, QHBoxLayout, QLineEdit, QMainWindow,
    QPushButton, QSizePolicy, QTextBrowser, QVBoxLayout,
    QWidget)

class Ui_MainWindow(object):
    def setupUi(self, MainWindow):
        if not MainWindow.objectName():
            MainWindow.setObjectName(u"MainWindow")
        MainWindow.resize(800, 600)
        self.centralwidget = QWidget(MainWindow)
        self.centralwidget.setObjectName(u"centralwidget")
        self.verticalLayout = QVBoxLayout(self.centralwidget)
        self.verticalLayout.setSpacing(0)
        self.verticalLayout.setObjectName(u"verticalLayout")
        self.verticalLayout.setContentsMargins(0, 0, 0, 0)
        self.horizontalLayout = QHBoxLayout()
        self.horizontalLayout.setObjectName(u"horizontalLayout")
        self.back_button = QPushButton(self.centralwidget)
        self.back_button.setObjectName(u"back_button")

        self.horizontalLayout.addWidget(self.back_button)

        self.refresh_button = QPushButton(self.centralwidget)
        self.refresh_button.setObjectName(u"refresh_button")

        self.horizontalLayout.addWidget(self.refresh_button)

        self.url_input = QLineEdit(self.centralwidget)
        self.url_input.setObjectName(u"url_input")

        self.horizontalLayout.addWidget(self.url_input)


        self.verticalLayout.addLayout(self.horizontalLayout)

        self.content = QTextBrowser(self.centralwidget)
        self.content.setObjectName(u"content")

        self.verticalLayout.addWidget(self.content)

        MainWindow.setCentralWidget(self.centralwidget)

        self.retranslateUi(MainWindow)

        QMetaObject.connectSlotsByName(MainWindow)
    # setupUi

    def retranslateUi(self, MainWindow):
        MainWindow.setWindowTitle(QCoreApplication.translate("MainWindow", u"Castor", None))
        self.back_button.setText(QCoreApplication.translate("MainWindow", u"\u2190", None))
        self.refresh_button.setText(QCoreApplication.translate("MainWindow", u"\u27f3", None))
    # retranslateUi

