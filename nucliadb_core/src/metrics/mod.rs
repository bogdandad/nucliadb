// Copyright (C) 2021 Bosutech XXI S.L.
//
// nucliadb is offered under the AGPL v3.0 and as commercial software.
// For commercial licensing, contact us at info@nuclia.com.
//
// AGPL:
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use std::fmt::Debug;

use prometheus_client::encoding;
use prometheus_client::registry::Registry;

use crate::{tracing, NodeResult};
// metrics
// Every metric must be define in its own module, which must fulfill the following requirements:
// - The name of the module must be the name of the name of the metric.
// - If the metric is called SomeName, then there must be a type 'SomeNameMetric' describing such
//   metric.
// - If the metric is called SomeName, a function 'register_some_name' must be defined and its job
//   is to recive a registry, register there the metric and return such metric.
// - If the metric is called SomeName, a struct 'SomeNameKey' must be defined.
// - If the metric is called SomeName, a struct 'SomeNameValue' must be defined.
pub mod request_time;

pub trait Metrics: Send + Sync {
    fn collect(&self) -> NodeResult<String>;
    fn record_request_time(
        &self,
        metric: request_time::RequestTimeKey,
        value: request_time::RequestTimeValue,
    );
}

pub struct PrometheusMetrics {
    registry: Registry,
    request_time_metric: request_time::RequestTimeMetric,
}

impl Default for PrometheusMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics for PrometheusMetrics {
    fn collect(&self) -> NodeResult<String> {
        let mut buf = String::new();
        encoding::text::encode(&mut buf, &self.registry)?;
        Ok(buf)
    }
    fn record_request_time(
        &self,
        metric: request_time::RequestTimeKey,
        value: request_time::RequestTimeValue,
    ) {
        self.request_time_metric
            .get_or_create(&metric)
            .observe(value);
    }
}

impl PrometheusMetrics {
    pub fn new() -> PrometheusMetrics {
        let mut registry = Registry::default();

        // This must be done for every metric
        let request_time_metric = request_time::register_request_time(&mut registry);

        PrometheusMetrics {
            registry,
            request_time_metric,
        }
    }
}

pub struct ConsoleLogMetrics;
impl ConsoleLogMetrics {
    fn record<Metric: Debug, Value: Debug>(&self, metric: Metric, value: Value) {
        tracing::debug!("{metric:?} : {value:?}")
    }
}
impl Metrics for ConsoleLogMetrics {
    fn collect(&self) -> NodeResult<String> {
        Ok(Default::default())
    }
    fn record_request_time(
        &self,
        metric: request_time::RequestTimeKey,
        value: request_time::RequestTimeValue,
    ) {
        self.record(metric, value)
    }
}

pub struct NoMetrics;
impl Metrics for NoMetrics {
    fn collect(&self) -> NodeResult<String> {
        Ok(Default::default())
    }
    fn record_request_time(
        &self,
        _: request_time::RequestTimeKey,
        _: request_time::RequestTimeValue,
    ) {
    }
}
