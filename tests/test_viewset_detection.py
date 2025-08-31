"""
Tests for viewset detection and command execution from viewset directories.

These tests verify that:
1. View commands work from viewset directories
2. Viewset auto-detection works correctly
3. --viewset parameter is not required when context is clear
4. Justfiles are created in viewset directories
"""

import tempfile
import pytest
import os
import sys
import shutil
from pathlib import Path
from unittest.mock import patch, MagicMock

# Add the scripts directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'scripts'))

import importlib.util
spec = importlib.util.spec_from_file_location("view_manager", os.path.join(os.path.dirname(__file__), '..', 'scripts', 'view-manager.py'))
view_manager = importlib.util.module_from_spec(spec)
spec.loader.exec_module(view_manager)

detect_current_viewset = view_manager.detect_current_viewset
load_viewsets_config = view_manager.load_viewsets_config
create_viewset_justfile = view_manager.create_viewset_justfile
ensure_viewset_justfiles = view_manager.ensure_viewset_justfiles
from test_utils import ViewsetConfigBuilder


class TestViewsetDetection:
    """Test viewset detection from working directory"""

    def setup_method(self):
        """Set up test environment"""
        self.temp_dir = tempfile.mkdtemp()
        self.original_home = os.environ.get('HOME')
        os.environ['HOME'] = self.temp_dir
        
        # Create test viewset config
        self.config_builder = ViewsetConfigBuilder(Path(self.temp_dir))
        self.config_builder.add_viewset("work", [
            {"name": "api-service", "url": "git@github.com:company/api-service.git"},
            {"name": "web-app", "url": "git@github.com:company/web-app.git"}
        ]).add_viewset("personal", [
            {"name": "my-project", "url": "git@github.com:me/my-project.git"}
        ]).build_config()
        
        # Create viewset directories
        self.work_dir = Path(self.temp_dir) / "src" / "work"
        self.personal_dir = Path(self.temp_dir) / "src" / "personal"
        self.work_views_dir = self.work_dir / "views"
        self.personal_views_dir = self.personal_dir / "views"
        
        self.work_views_dir.mkdir(parents=True)
        self.personal_views_dir.mkdir(parents=True)

    def teardown_method(self):
        """Clean up after each test"""
        if self.original_home:
            os.environ['HOME'] = self.original_home
        else:
            del os.environ['HOME']
        shutil.rmtree(self.temp_dir)

    def test_detect_viewset_from_work_directory(self):
        """Test detection when in work viewset directory"""
        with patch.object(view_manager.Path, 'cwd') as mock_cwd:
            mock_cwd.return_value = self.work_dir
            detected = detect_current_viewset()
            assert detected == "work"

    def test_detect_viewset_from_work_views_directory(self):
        """Test detection when in work viewset views directory"""
        with patch.object(view_manager.Path, 'cwd') as mock_cwd:
            mock_cwd.return_value = self.work_views_dir
            detected = detect_current_viewset()
            assert detected == "work"

    def test_detect_viewset_from_personal_directory(self):
        """Test detection when in personal viewset directory"""
        with patch.object(view_manager.Path, 'cwd') as mock_cwd:
            mock_cwd.return_value = self.personal_dir
            detected = detect_current_viewset()
            assert detected == "personal"

    def test_detect_viewset_from_view_subdirectory(self):
        """Test detection when in a specific view directory"""
        task_dir = self.work_views_dir / "TASK-123"
        task_dir.mkdir()

        with patch.object(view_manager.Path, 'cwd') as mock_cwd:
            mock_cwd.return_value = task_dir
            detected = detect_current_viewset()
            assert detected == "work"

    def test_no_detection_outside_src(self):
        """Test no detection when outside ~/src/ directory"""
        with patch.object(view_manager.Path, 'cwd') as mock_cwd:
            mock_cwd.return_value = Path(self.temp_dir) / "other"
            detected = detect_current_viewset()
            assert detected is None

    def test_no_detection_for_unconfigured_viewset(self):
        """Test no detection for directory that's not a configured viewset"""
        unconfigured_dir = Path(self.temp_dir) / "src" / "unknown"
        unconfigured_dir.mkdir(parents=True)

        with patch.object(view_manager.Path, 'cwd') as mock_cwd:
            mock_cwd.return_value = unconfigured_dir
            detected = detect_current_viewset()
            assert detected is None


class TestViewsetJustfiles:
    """Test justfile creation in viewset directories"""

    def setup_method(self):
        """Set up test environment"""
        self.temp_dir = tempfile.mkdtemp()
        self.original_home = os.environ.get('HOME')
        os.environ['HOME'] = self.temp_dir
        
        # Create test viewset config
        self.config_builder = ViewsetConfigBuilder(Path(self.temp_dir))
        self.config_builder.add_viewset("work", [
            {"name": "api-service", "url": "git@github.com:company/api-service.git"}
        ]).build_config()
        
        self.work_dir = Path(self.temp_dir) / "src" / "work"
        self.work_dir.mkdir(parents=True)

    def teardown_method(self):
        """Clean up after each test"""
        if self.original_home:
            os.environ['HOME'] = self.original_home
        else:
            del os.environ['HOME']
        shutil.rmtree(self.temp_dir)

    def test_create_viewset_justfile(self):
        """Test creating a justfile in viewset directory"""
        create_viewset_justfile(self.work_dir, "work")
        
        justfile_path = self.work_dir / "justfile"
        assert justfile_path.exists()
        
        content = justfile_path.read_text()
        assert "# Viewyard View Management for work viewset" in content
        assert "view *args:" in content
        assert "create view-name:" in content
        assert "list:" in content

    def test_ensure_viewset_justfiles_creates_missing(self):
        """Test that ensure_viewset_justfiles creates missing justfiles"""
        # Ensure justfile doesn't exist initially
        justfile_path = self.work_dir / "justfile"
        assert not justfile_path.exists()
        
        ensure_viewset_justfiles()
        
        # Should now exist
        assert justfile_path.exists()
        content = justfile_path.read_text()
        assert "work viewset" in content

    def test_ensure_viewset_justfiles_skips_existing(self):
        """Test that ensure_viewset_justfiles doesn't overwrite existing justfiles"""
        justfile_path = self.work_dir / "justfile"
        custom_content = "# Custom justfile content"
        justfile_path.write_text(custom_content)
        
        ensure_viewset_justfiles()
        
        # Should not be overwritten
        assert justfile_path.read_text() == custom_content


class TestViewCommandsFromViewsetDirectory:
    """Test that view commands work when run from viewset directories"""

    def setup_method(self):
        """Set up test environment"""
        self.temp_dir = tempfile.mkdtemp()
        self.original_home = os.environ.get('HOME')
        os.environ['HOME'] = self.temp_dir
        
        # Create test viewset config
        self.config_builder = ViewsetConfigBuilder(Path(self.temp_dir))
        self.config_builder.add_viewset("work", [
            {"name": "api-service", "url": "git@github.com:company/api-service.git"},
            {"name": "web-app", "url": "git@github.com:company/web-app.git"}
        ]).add_viewset("personal", [
            {"name": "my-project", "url": "git@github.com:me/my-project.git"}
        ]).build_config()
        
        self.work_dir = Path(self.temp_dir) / "src" / "work"
        self.work_views_dir = self.work_dir / "views"
        self.work_views_dir.mkdir(parents=True)

    def teardown_method(self):
        """Clean up after each test"""
        if self.original_home:
            os.environ['HOME'] = self.original_home
        else:
            del os.environ['HOME']
        shutil.rmtree(self.temp_dir)

    @patch.object(view_manager, 'input')
    @patch.object(view_manager.Path, 'cwd')
    def test_create_view_auto_detects_viewset(self, mock_cwd, mock_input):
        """Test that create_view auto-detects viewset from current directory"""

        mock_cwd.return_value = self.work_dir
        mock_input.return_value = "1"  # Select first repo

        # Should auto-detect work viewset and not require --viewset parameter
        view_manager.create_view("test-task")

        # Verify view was created in work viewset
        view_path = self.work_views_dir / "test-task"
        assert view_path.exists()

        repos_file = view_path / ".view-repos"
        assert repos_file.exists()
        assert "api-service" in repos_file.read_text()

    @patch.object(view_manager.Path, 'cwd')
    def test_load_workspace_auto_detects_viewset(self, mock_cwd):
        """Test that load_workspace auto-detects viewset from current directory"""

        mock_cwd.return_value = self.work_dir

        # Should auto-detect work viewset
        repos = view_manager.load_workspace()

        assert len(repos) == 2
        repo_names = [repo["name"] for repo in repos]
        assert "api-service" in repo_names
        assert "web-app" in repo_names
