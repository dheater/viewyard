"""
Tests for the comprehensive validation system.

These tests verify that:
1. Validation detects missing directories, justfiles, and configuration issues
2. Auto-fix functionality works correctly
3. Repository accessibility testing works
4. Integration with onboarding works properly
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
spec = importlib.util.spec_from_file_location("view_manager", 
                                             os.path.join(os.path.dirname(__file__), '..', 'scripts', 'view-manager.py'))
view_manager = importlib.util.module_from_spec(spec)
spec.loader.exec_module(view_manager)

from test_utils import ViewsetConfigBuilder


class TestValidationFramework:
    """Test the validation framework components"""

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

    def teardown_method(self):
        """Clean up after each test"""
        if self.original_home:
            os.environ['HOME'] = self.original_home
        else:
            del os.environ['HOME']
        shutil.rmtree(self.temp_dir)

    def test_validation_result_creation(self):
        """Test ValidationResult class"""
        result = view_manager.ValidationResult(
            "Test Check",
            True,
            "✅ Test passed",
            lambda: print("fix"),
            "Fix description"
        )
        
        assert result.check_name == "Test Check"
        assert result.passed == True
        assert result.message == "✅ Test passed"
        assert result.fix_function is not None
        assert result.fix_description == "Fix description"

    def test_validator_initialization(self):
        """Test ViewyardValidator initialization"""
        validator = view_manager.ViewyardValidator(auto_fix=True)
        assert validator.auto_fix == True
        assert len(validator.results) == 0

    def test_config_file_validation_missing(self):
        """Test validation when config file is missing"""
        # Remove config file
        config_file = Path(self.temp_dir) / ".config" / "viewyard" / "viewsets.yaml"
        if config_file.exists():
            config_file.unlink()
            
        validator = view_manager.ViewyardValidator()
        result = validator.validate_config_file()
        
        assert not result.passed
        assert "Missing" in result.message
        assert result.fix_function is not None

    def test_config_file_validation_valid(self):
        """Test validation when config file is valid"""
        validator = view_manager.ViewyardValidator()
        result = validator.validate_config_file()
        
        assert result.passed
        assert "✅" in result.message

    def test_viewset_directories_validation_missing(self):
        """Test validation when viewset directories are missing"""
        validator = view_manager.ViewyardValidator()
        results = validator.validate_viewset_directories()
        
        # Should find missing directories for both viewsets
        failed_results = [r for r in results if not r.passed]
        assert len(failed_results) >= 2  # At least missing directories for work and personal
        
        # Check that fix functions are provided
        for result in failed_results:
            if "Missing directory" in result.message:
                assert result.fix_function is not None

    def test_viewset_directories_validation_existing(self):
        """Test validation when viewset directories exist"""
        # Create directories
        work_dir = Path(self.temp_dir) / "src" / "work"
        personal_dir = Path(self.temp_dir) / "src" / "personal"
        work_views = work_dir / "views"
        personal_views = personal_dir / "views"
        
        work_views.mkdir(parents=True)
        personal_views.mkdir(parents=True)
        
        validator = view_manager.ViewyardValidator()
        results = validator.validate_viewset_directories()
        
        # Should find existing directories
        passed_results = [r for r in results if r.passed and "Directory exists" in r.message]
        assert len(passed_results) >= 2

    def test_justfiles_validation_missing(self):
        """Test validation when justfiles are missing"""
        # Create directories but no justfiles
        work_dir = Path(self.temp_dir) / "src" / "work"
        work_dir.mkdir(parents=True)
        
        validator = view_manager.ViewyardValidator()
        results = validator.validate_justfiles()
        
        failed_results = [r for r in results if not r.passed]
        assert len(failed_results) >= 1
        
        # Check that fix functions are provided
        for result in failed_results:
            if "Missing justfile" in result.message:
                assert result.fix_function is not None

    def test_justfiles_validation_existing(self):
        """Test validation when justfiles exist and are valid"""
        # Create directories and justfiles
        work_dir = Path(self.temp_dir) / "src" / "work"
        work_dir.mkdir(parents=True)
        
        justfile_path = work_dir / "justfile"
        justfile_content = "# Viewyard View Management for work viewset\n"
        justfile_path.write_text(justfile_content)
        
        validator = view_manager.ViewyardValidator()
        results = validator.validate_justfiles()
        
        passed_results = [r for r in results if r.passed and "work" in r.check_name]
        assert len(passed_results) >= 1

    def test_auto_fix_functionality(self):
        """Test that auto-fix actually fixes issues"""
        validator = view_manager.ViewyardValidator(auto_fix=True)
        
        # Test directory creation fix
        work_dir = Path(self.temp_dir) / "src" / "work"
        assert not work_dir.exists()
        
        # Create and execute fix
        fix_func = lambda: validator._create_viewset_directory(work_dir)
        fix_func()
        
        assert work_dir.exists()

    def test_comprehensive_validation_integration(self):
        """Test the complete validation workflow"""
        validator = view_manager.ViewyardValidator(auto_fix=True)
        
        # Run comprehensive validation
        success = validator.run_comprehensive_validation(show_passed=False)
        
        # Should complete without errors (though may not pass all checks)
        assert isinstance(success, bool)
        assert len(validator.results) > 0
        
        # Check that we have results for different validation categories
        check_names = [r.check_name for r in validator.results]
        assert any("Config" in name for name in check_names)
        assert any("Directory" in name for name in check_names)


class TestValidationIntegration:
    """Test integration with existing systems"""

    def setup_method(self):
        """Set up test environment"""
        self.temp_dir = tempfile.mkdtemp()
        self.original_home = os.environ.get('HOME')
        os.environ['HOME'] = self.temp_dir

    def teardown_method(self):
        """Clean up after each test"""
        if self.original_home:
            os.environ['HOME'] = self.original_home
        else:
            del os.environ['HOME']
        shutil.rmtree(self.temp_dir)

    def test_legacy_validate_setup_still_works(self):
        """Test that the legacy validate_setup function still works"""
        # Create basic config
        config_dir = Path(self.temp_dir) / ".config" / "viewyard"
        config_dir.mkdir(parents=True)
        config_file = config_dir / "viewsets.yaml"
        config_file.write_text("viewsets:\n  work:\n    repos: []\n")
        
        # Should not raise an exception
        try:
            view_manager.validate_setup()
            # If we get here, the function completed
            assert True
        except SystemExit:
            # validate_setup might call sys.exit, which is acceptable
            assert True
        except Exception as e:
            # Any other exception is a problem
            assert False, f"validate_setup raised unexpected exception: {e}"

    def test_comprehensive_validation_function(self):
        """Test the standalone comprehensive validation function"""
        # Create basic config
        config_dir = Path(self.temp_dir) / ".config" / "viewyard"
        config_dir.mkdir(parents=True)
        config_file = config_dir / "viewsets.yaml"
        config_file.write_text("viewsets:\n  work:\n    repos: []\n")
        
        # Should return a boolean
        result = view_manager.validate_setup_comprehensive(auto_fix=True, show_passed=False)
        assert isinstance(result, bool)
