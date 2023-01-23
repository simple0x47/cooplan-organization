import asyncio
import json
from amqp_api_client_py import amqp_input_api
from cooplan_integration_test_boilerplate import test
import amqp_config
import os
from pymongo import MongoClient
import time
from mongodb_config import ORGANIZATION_DATABASE, ORGANIZATION_COLLECTION, USER_DATABASE, USER_COLLECTION, \
    INVITATION_DATABASE, INVITATION_COLLECTION

TEST_TIMEOUT_AFTER_SECONDS_ENV = "TEST_TIMEOUT_AFTER_SECONDS"


async def read_organization_as_expected():
    client = MongoClient(os.environ.get(test.TEST_MONGODB_URI_ENV))

    EXAMPLE_ORGANIZATION = {
        "name": "Organization Test #1234",
        "country": "RO",
        "address": "Strada Exemplu Nr.15",
        "telephone": "+40712113640",
        "permissions": []
    }

    organization_insert = client[ORGANIZATION_DATABASE][
        ORGANIZATION_COLLECTION].insert_one(EXAMPLE_ORGANIZATION)

    EXAMPLE_INVITATION = {
        "code": "1234567890",
        "organization_id": organization_insert.inserted_id,
        "permissions": [],
        "created_at": int(time.time()),
        "expires_after": int(time.time()) + 3600,
    }

    client[INVITATION_DATABASE][INVITATION_COLLECTION].insert_one(EXAMPLE_INVITATION)

    EXAMPLE_USER = {
        "id": "example|12345678",
        "organizations": [
            {
                "organization_id": str(organization_insert.inserted_id),
                "permissions": []
            }
        ]
    }

    client[USER_DATABASE][USER_COLLECTION].insert_one(EXAMPLE_USER)

    REQUEST = {
        "header": {
            "element": "organization",
            "action": "read"
        },
        "organization_id": str(organization_insert.inserted_id),
    }

    test.init_request(REQUEST)

    input_api = amqp_input_api.AmqpInputApi(amqp_config.REQUEST_AMQP_CONFIG, amqp_config.RESPONSE_AMQP_CONFIG)

    timeout_after = int(os.environ.get(TEST_TIMEOUT_AFTER_SECONDS_ENV, 15))

    await asyncio.wait_for(input_api.connect(), timeout_after)

    serialized_result = await asyncio.wait_for(input_api.send_request(REQUEST), timeout_after)

    result = json.loads(serialized_result)

    print(f"result: {result}")

    assert ("Ok" in result)
    root = result["Ok"]

    assert ("organization" in root)
    assert ("users" in root)
    assert ("invitations" in root)

    organization = root["organization"]

    assert (len(organization["id"]) > 0)
    assert (EXAMPLE_ORGANIZATION["name"] == organization["name"])
    assert (EXAMPLE_ORGANIZATION["country"] == organization["country"])
    assert (EXAMPLE_ORGANIZATION["address"] == organization["address"])
    assert (EXAMPLE_ORGANIZATION["telephone"] == organization["telephone"])

    # Assert that user has been added to the organization.
    user = root["users"][0]

    assert (user is not None)
    assert ("id" in user)
    assert (len(user["id"]) > 0)

    # Assert that invitation code has been removed.

    invitation = root["invitations"][0]

    assert (invitation["code"] == EXAMPLE_INVITATION["code"])


def restore_mongodb_initial_state():
    if test.restore_initial_state(ORGANIZATION_DATABASE, ORGANIZATION_COLLECTION):
        print(f"successfully restored initial state for the '{ORGANIZATION_COLLECTION}' collection")
    else:
        print(f"failed to restore initial state for the '{ORGANIZATION_COLLECTION}' collection")

    if test.restore_initial_state(USER_DATABASE, USER_COLLECTION):
        print(f"successfully restored initial state for the '{USER_COLLECTION}' collection")
    else:
        print(f"failed to restore initial state for the '{USER_COLLECTION}' collection")

    if test.restore_initial_state(INVITATION_DATABASE, INVITATION_COLLECTION):
        print(f"successfully restored initial state for the '{INVITATION_COLLECTION}' collection")
    else:
        print(f"failed to restore initial state for the '{INVITATION_COLLECTION}' collection")


async def main():
    result_code = 0
    try:
        await read_organization_as_expected()
    except BaseException as e:
        print(f"Exception: {e}")
        result_code = 1
    finally:
        restore_mongodb_initial_state()

    exit(result_code)


if __name__ == "__main__":
    asyncio.run(main())
