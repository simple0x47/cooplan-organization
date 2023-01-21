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


async def join_organization_and_expect_it_as_response():
    client = MongoClient(os.environ.get(test.TEST_MONGODB_URI_ENV))

    EXAMPLE_ORGANIZATION = {
        "name": "Organization Test #1234",
        "country": "RO",
        "address": "Strada Exemplu Nr.15",
        "telephone": "+40712113640",
        "permissions": []
    }

    result = client[ORGANIZATION_COLLECTION][
        ORGANIZATION_DATABASE].insert_one(EXAMPLE_ORGANIZATION)

    EXAMPLE_REQUEST_CODE = "1234567890"

    client[INVITATION_DATABASE][INVITATION_COLLECTION].insert_one({
        "code": EXAMPLE_REQUEST_CODE,
        "organization_id": result.inserted_id,
        "permissions": [],
        "created_at": int(time.time()),
        "expires_at": int(time.time()) + 3600,
    })

    REQUEST = {
        "header": {
            "element": "organization",
            "action": "join"
        },
        "invitation_code": EXAMPLE_REQUEST_CODE
    }

    test.init_request(REQUEST)

    input_api = amqp_input_api.AmqpInputApi(amqp_config.REQUEST_AMQP_CONFIG, amqp_config.RESPONSE_AMQP_CONFIG)

    timeout_after = int(os.environ.get(TEST_TIMEOUT_AFTER_SECONDS_ENV, 15))

    await asyncio.wait_for(input_api.connect(), timeout_after)

    serialized_result = await asyncio.wait_for(input_api.send_request(REQUEST), timeout_after)

    result = json.loads(serialized_result)

    if not ("Ok" in result):
        print(f"result: {result}")

    assert ("Ok" in result)
    organization = result["Ok"]

    assert ("id" in organization)
    assert ("name" in organization)
    assert ("country" in organization)
    assert ("address" in organization)
    assert ("telephone" in organization)
    assert (len(organization["id"]) > 0)
    assert (REQUEST["name"] == EXAMPLE_ORGANIZATION["name"])
    assert (REQUEST["country"] == EXAMPLE_ORGANIZATION["country"])
    assert (REQUEST["address"] == EXAMPLE_ORGANIZATION["address"])
    assert (REQUEST["telephone"] == EXAMPLE_ORGANIZATION["telephone"])

    # Assert that user has been added to the organization.
    user = client[USER_DATABASE][USER_COLLECTION].find_one({"organizations.organization_id": organization["id"]})

    assert (user is not None)
    assert ("id" in user)
    assert (len(user["id"]) > 0)

    # Assert that invitation code has been removed.
    invitation_code = client[INVITATION_DATABASE][INVITATION_COLLECTION].find_one({"code": EXAMPLE_REQUEST_CODE})

    assert (invitation_code is None)


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
        await join_organization_and_expect_it_as_response()
    except BaseException as e:
        print(f"Exception: {e}")
        result_code = 1
    finally:
        restore_mongodb_initial_state()

    exit(result_code)


if __name__ == "__main__":
    asyncio.run(main())
