import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("deployment", ROOT / "scripts/verify-deployment.py")
deployment = importlib.util.module_from_spec(spec)
spec.loader.exec_module(deployment)


class DeploymentChecks(unittest.TestCase):
    def test_shared_crate_is_copied_before_cooking_in_every_image(self):
        for path in ("Containerfile", "blueprints/lms/Dockerfile", "blueprints/portfolio/Dockerfile"):
            source = (ROOT / path).read_text(encoding="utf-8")
            self.assertLess(source.index("COPY crates/blueprint-ai /app/crates/blueprint-ai"),
                            source.index("RUN cargo chef cook"))

    def test_old_renderer_and_fallbacks_do_not_pass(self):
        for body in ("**old response**", '<div class="rullst-ai-prose"><p>Offline assistant</p></div>',
                     '<div class="rullst-ai-prose"><p>AI temporarily unavailable.</p></div>'):
            with self.assertRaises(RuntimeError):
                deployment.rendered_online(body)
        deployment.rendered_online('<div class="rullst-ai-prose"><p><strong>Rullst</strong></p></div>')

    def test_no_redirect_can_forward_credentials(self):
        self.assertIsNone(deployment.NoRedirect().redirect_request(
            None, None, 302, "redirect", {}, "https://invalid.example"))

    def test_required_configuration_fails_closed(self):
        properties = {"template": {"containers": [{"env": []}]}}
        with self.assertRaisesRegex(RuntimeError, "NEXUS_ADMIN_PASSWORD"):
            deployment.configuration("rullst-showcase", properties)
        properties["template"]["containers"][0]["env"] = [
            {"name": "NEXUS_ADMIN_PASSWORD", "value": "test-password-long-enough"},
            {"name": "GROQ_API_KEY", "value": "mock_test"},
        ]
        with self.assertRaisesRegex(RuntimeError, "real server-side Groq"):
            deployment.configuration("rullst-showcase", properties)

    def test_secret_reference_is_resolved_without_writes(self):
        properties = {"template": {"containers": [{"env": [
            {"name": "NEXUS_ADMIN_PASSWORD", "secretRef": "admin-password"},
            {"name": "GROQ_API_KEY", "value": "synthetic-test-only"},
        ]}]}}
        with patch.object(deployment, "azure", return_value="test-password-long-enough") as read:
            user, _ = deployment.configuration("rullst-lms", properties)
            self.assertEqual(user, "admin")
            self.assertEqual(read.call_args.args[1:4], ("secret", "list", "--show-values"))


if __name__ == "__main__":
    unittest.main()
