import asyncio
import json
from amqp_api_client_py import amqp_input_api
from cooplan_integration_test_boilerplate import test
import amqp_config
import os

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


async def main():
    asyncio.run(create_organization_and_expect_it_as_response())


if __name__ == "__main__":
    asyncio.run(main())
