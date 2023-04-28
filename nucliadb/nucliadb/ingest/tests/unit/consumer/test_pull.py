# Copyright (C) 2021 Bosutech XXI S.L.
#
# nucliadb is offered under the AGPL v3.0 and as commercial software.
# For commercial licensing, contact us at info@nuclia.com.
#
# AGPL:
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as
# published by the Free Software Foundation, either version 3 of the
# License, or (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program. If not, see <http://www.gnu.org/licenses/>.
from unittest import mock
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from aiohttp.web import Response

from nucliadb.ingest.consumer.pull import PullWorker, check_proxy_telemetry_headers


@pytest.fixture(scope="function")
def errors():
    with mock.patch("nucliadb.ingest.consumer.pull.errors") as errors:
        yield errors


def test_check_proxy_telemetry_headers_ok(errors):
    resp = Response(
        headers={"x-b3-traceid": "foo", "x-b3-spanid": "bar", "x-b3-sampled": "baz"}
    )
    check_proxy_telemetry_headers(resp)

    errors.capture_exception.assert_not_called()


class TestPullWorker:
    """
    It's a complex class so this might get a little messy with mocks

    It should be refactor at some point and these tests be rewritten/removed
    """

    @pytest.fixture()
    def processor(self):
        processor = AsyncMock()
        with patch("nucliadb.ingest.consumer.pull.Processor", return_value=processor):
            yield processor

    @pytest.fixture()
    def nats_conn(self):
        conn = MagicMock()
        conn.jetstream.return_value = AsyncMock()
        conn.drain = AsyncMock()
        conn.close = AsyncMock()
        with patch("nucliadb.ingest.consumer.pull.nats.connect", return_value=conn):
            yield conn

    @pytest.fixture()
    def worker(self, processor):
        yield PullWorker(
            driver=AsyncMock(),
            partition="1",
            storage=AsyncMock(),
            pull_time=100,
            zone="zone",
            nuclia_cluster_url="nuclia_cluster_url",
            nuclia_public_url="nuclia_public_url",
            audit=None,
            target="target",
            group="group",
            stream="stream",
            onprem=False,
        )

    async def test_lifecycle(self, worker: PullWorker, processor, nats_conn):
        await worker.initialize()

        assert worker.processor == processor
        assert worker.nc == nats_conn

        nats_conn.jetstream.assert_called_once()
        nats_conn.jetstream().subscribe.assert_called_once()

        await worker.finalize()

        nats_conn.jetstream().subscribe.return_value.drain.assert_called_once()

    async def test_reconnect(self, worker: PullWorker, processor, nats_conn):
        await worker.initialize()
        await worker.reconnected_cb()

        assert nats_conn.jetstream().subscribe.call_count == 2
