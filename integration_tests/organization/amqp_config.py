REQUEST_AMQP_CONFIG = {
    "queue": {
        "name": "organization",
    },
    "channel": {
        "publish": {
            "mandatory": False,
            "immediate": False,
            "timeout": None
        }
    }
}

RESPONSE_AMQP_CONFIG = {
    "queue": {
        "name": "",
        "passive": False,
        "durable": False,
        "exclusive": False,
        "auto_delete": True,
        "nowait": False,
        "arguments": {}
    },
    "channel": {
        "consume": {
            "no_ack": False,
            "exclusive": False,
            "arguments": {},
            "consumer_tag": None,
            "timeout": None
        }
    }
}