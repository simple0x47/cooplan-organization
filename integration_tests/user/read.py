import asyncio
import json
from amqp_api_client_py import amqp_input_api
from cooplan_integration_test_boilerplate import test
import amqp_config
import os
from pymongo import MongoClient
from mongodb_config import ORGANIZATION_DATABASE, ORGANIZATION_COLLECTION, USER_DATABASE, USER_COLLECTION
TEST_TIMEOUT_AFTER_SECONDS_ENV = "TEST_TIMEOUT_AFTER_SECONDS"


async def read_user_as_expected():
    client = MongoClient(os.environ.get(test.TEST_MONGODB_URI_ENV))

    EXAMPLE_ORGANIZATION = {
        "name": "Organization Test #5319",
        "country": "RO",
        "address": "Strada Exemplu Nr.15",
        "telephone": "+40712111340",
        "permissions": []
    }

    organization_insert = client[ORGANIZATION_DATABASE][
        ORGANIZATION_COLLECTION].insert_one(EXAMPLE_ORGANIZATION)

    EXAMPLE_USER = {
        "id": "example|12345678",
        "organizations": [
            {
                "organization_id": str(organization_insert.inserted_id),
                "permissions": ["read:organization", "update:user"]
            }
        ]
    }

    client[USER_DATABASE][USER_COLLECTION].insert_one(EXAMPLE_USER)

    REQUEST = {
        "header": {
            "element": "user",
            "action": "read"
        },
        "user_id": str(EXAMPLE_USER["id"]),
    }

    test.init_request(REQUEST)

    input_api = amqp_input_api.AmqpInputApi(amqp_config.REQUEST_AMQP_CONFIG, amqp_config.RESPONSE_AMQP_CONFIG)

    timeout_after = int(os.environ.get(TEST_TIMEOUT_AFTER_SECONDS_ENV, 15))

    await asyncio.wait_for(input_api.connect(), timeout_after)

    serialized_result = await asyncio.wait_for(input_api.send_request(REQUEST), timeout_after)

    result = json.loads(serialized_result)

    print(f"result: {result}")

    assert ("Ok" in result)
    user = result["Ok"]

    assert ("id" in user)
    assert ("organizations" in user)
    assert (user["organizations"] is not None)
    assert (len(user["organizations"]) == 1)
    assert (EXAMPLE_USER["id"] == user["id"])

    user_organization = user["organizations"][0]

    assert (len(user_organization["organization_id"]) > 0)
    assert (str(organization_insert.inserted_id) == user_organization["organization_id"])

    assert (len(user_organization["permissions"]) == 2)
    assert ("read:organization" in user_organization["permissions"])
    assert ("update:user" in user_organization["permissions"])


def restore_mongodb_initial_state():
    if test.restore_initial_state(ORGANIZATION_DATABASE, ORGANIZATION_COLLECTION):
        print(f"successfully restored initial state for the '{ORGANIZATION_COLLECTION}' collection")
    else:
        print(f"failed to restore initial state for the '{ORGANIZATION_COLLECTION}' collection")

    if test.restore_initial_state(USER_DATABASE, USER_COLLECTION):
        print(f"successfully restored initial state for the '{USER_COLLECTION}' collection")
    else:
        print(f"failed to restore initial state for the '{USER_COLLECTION}' collection")

async def main():
    result_code = 0
    try:
        await read_user_as_expected()
    except BaseException as e:
        print(f"Exception: {e}")
        result_code = 1
    finally:
        restore_mongodb_initial_state()

    exit(result_code)


if __name__ == "__main__":
    asyncio.run(main())
