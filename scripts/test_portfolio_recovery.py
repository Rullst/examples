import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("recovery", Path(__file__).with_name("repair-portfolio-deployment.py"))
recovery = importlib.util.module_from_spec(spec)
spec.loader.exec_module(recovery)


class PortfolioRecoveryChecks(unittest.TestCase):
    def test_no_reads_or_writes_without_explicit_approval(self):
        with patch.object(recovery.verify, "azure") as read, patch.object(recovery, "azure_write") as write:
            with self.assertRaises(RuntimeError):
                recovery.repair(False)
            read.assert_not_called()
            write.assert_not_called()

    def test_missing_portfolio_provider_prevents_any_writes(self):
        source = {"template": {"containers": [{"env": [
            {"name": "NEXUS_ADMIN_PASSWORD", "value": "test-password-long-enough"},
            {"name": "GROQ_API_KEY", "value": "synthetic-test-key"},
        ]}]}}
        target = {"template": {"containers": [{"name": "portfolio", "env": []}]}}
        with patch.object(recovery.verify, "azure", side_effect=[{"properties": source}, {"properties": target}]), \
                patch.object(recovery, "azure_write") as write:
            with self.assertRaisesRegex(RuntimeError, "Groq"):
                recovery.repair(True)
            write.assert_not_called()

    def test_only_destination_image_and_password_are_updated(self):
        password = "synthetic-test-password-long-enough"
        target = {"template": {"containers": [{"name": "portfolio", "env": [
            {"name": "NEXUS_ADMIN_USERNAME", "value": "existing-user"},
        ]}]}}
        with patch.object(recovery.verify, "azure", return_value={"properties": target}), \
                patch.object(recovery.verify, "configuration", return_value=("existing-user", password)), \
                patch.object(recovery, "azure_write") as write:
            recovery.repair(True)
            self.assertEqual(write.call_count, 2)
            self.assertEqual(write.call_args_list[0].args,
                             ("secret", "set", "--secrets", f"{recovery.SECRET}={password}"))
            self.assertEqual(write.call_args_list[1].args,
                             ("update", "--image", recovery.IMAGE, "--set-env-vars",
                              f"NEXUS_ADMIN_PASSWORD=secretref:{recovery.SECRET}", "--container-name", "portfolio"))
            self.assertEqual(target["template"]["containers"][0]["env"],
                             [{"name": "NEXUS_ADMIN_USERNAME", "value": "existing-user"}])

    def test_write_helper_is_restricted_to_portfolio_and_captures_output(self):
        with patch.object(recovery.subprocess, "run") as run:
            run.return_value.returncode = 0
            recovery.azure_write("secret", "set", "--secrets", "synthetic=value")
            argv = run.call_args.args[0]
            self.assertEqual(argv[argv.index("--name") + 1], "rullst-portfolio")
            self.assertEqual(argv[argv.index("--output") + 1], "none")
            self.assertTrue(run.call_args.kwargs["capture_output"])


if __name__ == "__main__":
    unittest.main()
