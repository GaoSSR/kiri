use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerInfo {
    pub name: String,
    pub image: String,
}

pub fn batch_docker_info() -> HashMap<u16, DockerInfo> {
    let output = match Command::new("docker")
        .args(["ps", "--format", "{{.Ports}}\t{{.Names}}\t{{.Image}}"])
        .output()
    {
        Ok(output) => output,
        Err(_) => return HashMap::new(),
    };

    if !output.status.success() {
        return HashMap::new();
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    parse_docker_ps(&raw)
}

pub fn parse_docker_ps(raw: &str) -> HashMap<u16, DockerInfo> {
    let mut map = HashMap::new();

    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 3 {
            continue;
        }

        let ports = fields[0];
        let name = fields[1];
        let image = fields[2];
        if ports.is_empty() || name.is_empty() || image.is_empty() {
            continue;
        }

        let mut line_seen = HashMap::new();
        for segment in ports.split(',') {
            if let Some(port) = parse_host_port(segment.trim()) {
                line_seen.entry(port).or_insert_with(|| DockerInfo {
                    name: name.to_string(),
                    image: image.to_string(),
                });
            }
        }

        for (port, info) in line_seen {
            map.entry(port).or_insert(info);
        }
    }

    map
}

pub fn detect_framework_from_image(image: &str) -> &'static str {
    let image = image.to_ascii_lowercase();

    if image.contains("postgres") {
        "PostgreSQL"
    } else if image.contains("redis") {
        "Redis"
    } else if image.contains("mysql") || image.contains("mariadb") {
        "MySQL"
    } else if image.contains("mongo") {
        "MongoDB"
    } else if image.contains("nginx") {
        "nginx"
    } else if image.contains("localstack") {
        "LocalStack"
    } else if image.contains("rabbitmq") {
        "RabbitMQ"
    } else if image.contains("kafka") {
        "Kafka"
    } else if image.contains("elasticsearch") || image.contains("opensearch") {
        "Elasticsearch"
    } else if image.contains("minio") {
        "MinIO"
    } else {
        "Docker"
    }
}

fn parse_host_port(segment: &str) -> Option<u16> {
    let (host, _) = segment.split_once("->")?;
    let (_, port_raw) = host.rsplit_once(':')?;
    if port_raw.is_empty() || !port_raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let port = port_raw.parse::<u32>().ok()?;
    if port == 0 || port > u16::MAX as u32 {
        return None;
    }

    Some(port as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ipv4_host_port_mapping() {
        let raw = "0.0.0.0:5432->5432/tcp\tbackend-postgres-1\tpostgres:16\n";

        let map = parse_docker_ps(raw);

        assert_eq!(
            map.get(&5432),
            Some(&DockerInfo {
                name: "backend-postgres-1".to_string(),
                image: "postgres:16".to_string(),
            })
        );
    }

    #[test]
    fn parses_multiple_ports_from_same_container_line() {
        let raw =
            "0.0.0.0:4566->4566/tcp, 0.0.0.0:4510->4510/tcp\tlocalstack\tlocalstack/localstack:latest\n";

        let map = parse_docker_ps(raw);

        assert_eq!(
            map.get(&4566).map(|info| info.name.as_str()),
            Some("localstack")
        );
        assert_eq!(
            map.get(&4510).map(|info| info.name.as_str()),
            Some("localstack")
        );
    }

    #[test]
    fn parses_ipv6_host_port_mapping() {
        let raw = ":::6379->6379/tcp\tbackend-redis-1\tredis:7\n";

        let map = parse_docker_ps(raw);

        assert_eq!(
            map.get(&6379).map(|info| info.image.as_str()),
            Some("redis:7")
        );
    }

    #[test]
    fn ignores_empty_and_incomplete_lines() {
        let raw = "\n0.0.0.0:5432->5432/tcp\tmissing-image\nmissing-tabs\n";

        let map = parse_docker_ps(raw);

        assert!(map.is_empty());
    }

    #[test]
    fn keeps_first_mapping_for_duplicate_host_port() {
        let raw = "\
0.0.0.0:5432->5432/tcp\tfirst-postgres\tpostgres:15
0.0.0.0:5432->5432/tcp\tsecond-postgres\tpostgres:16
";

        let map = parse_docker_ps(raw);

        assert_eq!(
            map.get(&5432).map(|info| info.name.as_str()),
            Some("first-postgres")
        );
    }

    #[test]
    fn detects_framework_from_known_images() {
        let cases = [
            ("postgres:16", "PostgreSQL"),
            ("redis:7", "Redis"),
            ("mysql:8", "MySQL"),
            ("mariadb:11", "MySQL"),
            ("mongo:7", "MongoDB"),
            ("nginx:alpine", "nginx"),
            ("localstack/localstack:latest", "LocalStack"),
            ("rabbitmq:3", "RabbitMQ"),
            ("bitnami/kafka:latest", "Kafka"),
            ("elasticsearch:8", "Elasticsearch"),
            ("opensearchproject/opensearch:2", "Elasticsearch"),
            ("minio/minio:latest", "MinIO"),
            ("custom/app:latest", "Docker"),
        ];

        for (image, expected) in cases {
            assert_eq!(detect_framework_from_image(image), expected);
        }
    }
}
