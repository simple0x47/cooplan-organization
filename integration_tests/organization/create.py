import asyncio
import json
from amqp_api_client_py import amqp_input_api
from cooplan_integration_test_boilerplate import test
import amqp_config
import os
from pymongo import MongoClient

from mongodb_config import ORGANIZATION_DATABASE, ORGANIZATION_COLLECTION, USER_DATABASE, USER_COLLECTION

TEST_TIMEOUT_AFTER_SECONDS_ENV = "TEST_TIMEOUT_AFTER_SECONDS"


async def create_organization_and_expect_it_as_response():
    REQUEST = {
        "header": {
            "element": "organization",
            "action": "create"
        },
        "name": "Organization Test #1234",
        "country": "RO",
        "address": "Strada Exemplu Nr.15",
        "telephone": "+40753313640"
    }

    test.init_request(REQUEST)

    input_api = amqp_input_api.AmqpInputApi(amqp_config.REQUEST_AMQP_CONFIG, amqp_config.RESPONSE_AMQP_CONFIG)

    timeout_after = int(os.environ.get(TEST_TIMEOUT_AFTER_SECONDS_ENV, 15))

    await asyncio.wait_for(input_api.connect(), timeout_after)

    serialized_result = await asyncio.wait_for(input_api.send_request(REQUEST), timeout_after)

    result = json.loads(serialized_result)

    assert ("Ok" in result)
    organization = result["Ok"]

    assert ("id" in organization)
    assert ("name" in organization)
    assert ("country" in organization)
    assert ("address" in organization)
    assert ("telephone" in organization)
    assert (len(organization["id"]) > 0)
    assert (REQUEST["name"] == organization["name"])
    assert (REQUEST["country"] == organization["country"])
    assert (REQUEST["address"] == organization["address"])
    assert (REQUEST["telephone"] == organization["telephone"])

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
    try:
        await create_organization_and_expect_it_as_response()
    except:
        pass
    finally:
        restore_mongodb_initial_state()


if __name__ == "__main__":
    asyncio.run(main())
